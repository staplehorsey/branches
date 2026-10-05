//! Worlds live in a git repository. Every file an architect (or a person)
//! might want to read is plain JSON in a stable layout:
//!
//! ```text
//! worlds/<id>/world.json        manifest, biome included
//! worlds/<id>/rooms/<x>,<z>.json rooms: chambers, features, things, doors,
//!                                visitor book, gaze and use counts
//! worlds/<id>/heat.json          how much each cell has been explored
//! worlds/<id>/versions.json      tagged versions, newest first
//! worlds/<id>/paths.json         recent path signatures of wanderers
//! worlds/<id>/log.json           the world's own guestbook
//! .branches/players.json         private: never committed
//! ```
//!
//! Builds are commits. Every few builds a world gets a version tag in Go's
//! submodule style, `worlds/<id>/v0.N.0`; older versions are read straight
//! from those tags when the far reaches of a world show them.

use crate::model::*;
use git2::{IndexAddOption, Repository, Signature};
use serde::{Serialize, de::DeserializeOwned};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub struct Store {
    pub root: PathBuf,
    repo: Mutex<Repository>,
    written: Mutex<HashMap<PathBuf, u64>>,
    old: Mutex<HashMap<(String, String), Option<Room>>>,
    old_worlds: Mutex<HashMap<String, Option<WorldManifest>>>,
}

const README: &str = "# Branches worlds\n\nThis folder is a git repository kept by the Branches app. Each world is a\nfolder under `worlds/`. Every architect build is a commit, and every few\nbuilds a world gets a version tag (`worlds/<id>/v0.N.0`). Push it anywhere to\nback it up or share it.\n";

fn digest(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ *b as u64).wrapping_mul(0x100_0000_01b3))
}

fn read_json<T: DeserializeOwned>(path: &Path) -> Option<T> {
    let bytes = std::fs::read(path).ok()?;
    match serde_json::from_slice(&bytes) {
        Ok(v) => Some(v),
        Err(e) => {
            tracing::warn!("skipping {}: {e}", path.display());
            None
        }
    }
}

impl Store {
    pub fn open(root: &Path) -> anyhow_lite::Result<Self> {
        std::fs::create_dir_all(root)?;
        let repo = match Repository::open(root) {
            Ok(r) => r,
            Err(_) => Repository::init(root)?,
        };
        let ignore = root.join(".gitignore");
        if !ignore.exists() {
            std::fs::write(&ignore, ".branches/\n*.tmp\n")?;
        }
        if !root.join("README.md").exists() {
            std::fs::write(root.join("README.md"), README)?;
        }
        std::fs::create_dir_all(root.join(".branches"))?;
        Ok(Store { root: root.to_path_buf(), repo: Mutex::new(repo), written: Mutex::new(HashMap::new()), old: Mutex::new(HashMap::new()), old_worlds: Mutex::new(HashMap::new()) })
    }

    fn world_dir(&self, id: &str) -> PathBuf {
        self.root.join("worlds").join(id)
    }

    pub fn load(&self) -> Universe {
        let mut uni = Universe::default();
        // Older hosts kept everything in one file; bring it across once.
        let legacy = self.root.join("universe.json");
        if legacy.exists() && !self.root.join("worlds").exists() {
            if let Some(u) = read_json::<Universe>(&legacy) {
                uni = u;
                let _ = std::fs::rename(&legacy, self.root.join(".branches/universe.legacy.json"));
            }
        }
        if let Ok(entries) = std::fs::read_dir(self.root.join("worlds")) {
            for e in entries.flatten() {
                let dir = e.path();
                let Some(manifest) = read_json::<WorldManifest>(&dir.join("world.json")) else { continue };
                let mut w = World::new(manifest);
                if let Ok(rooms) = std::fs::read_dir(dir.join("rooms")) {
                    for f in rooms.flatten() {
                        if let Some(r) = read_json::<Room>(&f.path()) {
                            w.rooms.insert(addr(r.x, r.z), r);
                        }
                    }
                }
                w.heat = read_json(&dir.join("heat.json")).unwrap_or_default();
                w.versions = read_json(&dir.join("versions.json")).unwrap_or_default();
                w.paths = read_json(&dir.join("paths.json")).unwrap_or_default();
                w.log = read_json(&dir.join("log.json")).unwrap_or_default();
                if let Some(meta) = read_json::<BTreeMap<String, f64>>(&dir.join(".meta.json")) {
                    w.builds_since_tag = meta.get("builds_since_tag").copied().unwrap_or(0.0) as u32;
                    w.fresh_cells = meta.get("fresh_cells").copied().unwrap_or(0.0) as u32;
                    w.pace = meta.get("pace").copied().unwrap_or(0.0) as f32;
                }
                uni.worlds.insert(w.manifest.id.clone(), w);
            }
        }
        if let Some(players) = read_json(&self.root.join(".branches/players.json")) {
            uni.players = players;
        }
        uni
    }

    fn put<T: Serialize + ?Sized>(&self, path: PathBuf, value: &T) -> std::io::Result<bool> {
        let bytes = serde_json::to_vec_pretty(value).map_err(std::io::Error::other)?;
        let d = digest(&bytes);
        let mut written = self.written.lock().unwrap();
        if written.get(&path) == Some(&d) {
            return Ok(false);
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, &bytes)?;
        std::fs::rename(&tmp, &path)?;
        written.insert(path, d);
        Ok(true)
    }

    /// Write everything that changed. Returns whether anything did.
    pub fn save(&self, uni: &Universe) -> std::io::Result<bool> {
        let mut changed = false;
        for w in uni.worlds.values() {
            let dir = self.world_dir(&w.manifest.id);
            changed |= self.put(dir.join("world.json"), &w.manifest)?;
            for (key, r) in &w.rooms {
                changed |= self.put(dir.join("rooms").join(format!("{key}.json")), r)?;
            }
            if !w.heat.is_empty() {
                changed |= self.put(dir.join("heat.json"), &w.heat)?;
            }
            if !w.versions.is_empty() {
                changed |= self.put(dir.join("versions.json"), &w.versions)?;
            }
            if !w.paths.is_empty() {
                changed |= self.put(dir.join("paths.json"), &w.paths)?;
            }
            if !w.log.is_empty() {
                changed |= self.put(dir.join("log.json"), &w.log)?;
            }
            let meta: BTreeMap<&str, f64> = [("builds_since_tag", w.builds_since_tag as f64), ("fresh_cells", w.fresh_cells as f64), ("pace", w.pace as f64)].into();
            changed |= self.put(dir.join(".meta.json"), &meta)?;
        }
        self.put(self.root.join(".branches/players.json"), &uni.players)?;
        Ok(changed)
    }

    /// Commit the working tree if it differs from HEAD.
    pub fn commit(&self, message: &str) -> Result<Option<git2::Oid>, git2::Error> {
        let repo = self.repo.lock().unwrap();
        let mut index = repo.index()?;
        index.add_all(["worlds", ".gitignore", "README.md"], IndexAddOption::DEFAULT, None)?;
        index.update_all(["worlds"], None)?;
        index.write()?;
        let tree_id = index.write_tree()?;
        let parent = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
        if parent.as_ref().is_some_and(|p| p.tree_id() == tree_id) {
            return Ok(None);
        }
        let tree = repo.find_tree(tree_id)?;
        let sig = Signature::now("Branches architect", "architect@branches.local")?;
        let parents: Vec<&git2::Commit> = parent.iter().collect();
        Ok(Some(repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)?))
    }

    pub fn tag(&self, name: &str, message: &str) -> Result<(), git2::Error> {
        let repo = self.repo.lock().unwrap();
        let head = repo.head()?.peel(git2::ObjectType::Commit)?;
        let sig = Signature::now("Branches architect", "architect@branches.local")?;
        repo.tag(name, &head, &sig, message, true)?;
        Ok(())
    }

    fn read_at<T: DeserializeOwned>(&self, tag: &str, path: &str) -> Option<T> {
        let repo = self.repo.lock().unwrap();
        let obj = repo.revparse_single(&format!("refs/tags/{tag}")).ok()?;
        let tree = obj.peel_to_commit().ok()?.tree().ok()?;
        let entry = tree.get_path(Path::new(path)).ok()?;
        let blob = repo.find_blob(entry.id()).ok()?;
        serde_json::from_slice(blob.content()).ok()
    }

    /// A room as it was at a version tag (None if it was untouched then).
    pub fn room_at(&self, tag: &str, world: &str, key: &str) -> Option<Room> {
        let k = (tag.to_string(), format!("{world}/{key}"));
        if let Some(hit) = self.old.lock().unwrap().get(&k) {
            return hit.clone();
        }
        let room = self.read_at(tag, &format!("worlds/{world}/rooms/{key}.json"));
        self.old.lock().unwrap().insert(k, room.clone());
        room
    }

    /// A world's manifest (its biome) as it was at a version tag.
    pub fn world_at(&self, tag: &str, world: &str) -> Option<WorldManifest> {
        if let Some(hit) = self.old_worlds.lock().unwrap().get(tag) {
            return hit.clone();
        }
        let m = self.read_at(tag, &format!("worlds/{world}/world.json"));
        self.old_worlds.lock().unwrap().insert(tag.to_string(), m.clone());
        m
    }
}

/// A tiny error type so `open` can use `?` on both io and git errors.
pub mod anyhow_lite {
    pub type Result<T> = std::result::Result<T, Error>;
    #[derive(Debug)]
    pub struct Error(pub String);
    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(&self.0)
        }
    }
    impl From<std::io::Error> for Error {
        fn from(e: std::io::Error) -> Self {
            Error(e.to_string())
        }
    }
    impl From<git2::Error> for Error {
        fn from(e: git2::Error) -> Self {
            Error(e.to_string())
        }
    }
}

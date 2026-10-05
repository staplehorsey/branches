//! Shared host state: the universe plus live multiplayer plumbing.

use crate::architect::{self, Architect, Budget};
use crate::store::Store;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use crate::procgen;
use crate::model::*;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::{broadcast, mpsc};

pub struct Config {
    pub port: u16,
    pub data_dir: PathBuf,
    pub client_dir: PathBuf,
    /// Seconds of attention the first growth in a room costs.
    pub pace: f64,
    pub admin_key: Option<String>,
    /// Running as the desktop app: local only, opens the browser, can quit.
    pub app: bool,
}

impl Config {
    pub fn from_env() -> Self {
        let env = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
        let app = std::env::args().any(|a| a == "--app") || env("BRANCHES_APP").is_some();
        // Inside Branches.app the client sits in Contents/Resources/client.
        let bundled = std::env::current_exe().ok().and_then(|e| e.parent().map(|d| d.join("../Resources/client"))).filter(|p| p.join("index.html").exists());
        let client_dir = env("CLIENT_DIR").map(PathBuf::from).or(bundled).unwrap_or_else(|| {
            ["client", "../client"].iter().map(PathBuf::from).find(|p| p.join("index.html").exists()).unwrap_or_else(|| "client".into())
        });
        let app_data = env("HOME").map(|h| {
            if cfg!(target_os = "macos") {
                PathBuf::from(h).join("Library/Application Support/Branches")
            } else {
                PathBuf::from(h).join(".local/share/branches")
            }
        });
        let default_data = if app { app_data.unwrap_or_else(|| "data".into()) } else { "data".into() };
        Config {
            port: env("PORT").and_then(|p| p.parse().ok()).unwrap_or(if app { 7878 } else { 8080 }),
            data_dir: env("DATA_DIR").map(PathBuf::from).unwrap_or(default_data),
            client_dir,
            pace: env("ARCHITECT_PACE").and_then(|p| p.parse().ok()).unwrap_or(if app { 30.0 } else { 40.0 }),
            admin_key: env("ADMIN_KEY"),
            app,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Presence {
    pub id: String,
    pub name: String,
    pub color: String,
    pub p: [f32; 3],
    pub ry: f32,
    pub room: Option<String>,
    pub ch: Option<u32>,
    pub last_move: std::time::Instant,
    pub room_since: std::time::Instant,
    pub v: Option<String>,
    pub conn: u64,
}

pub type Msg = Arc<str>;

pub struct App {
    pub cfg: Config,
    pub uni: Mutex<Universe>,
    pub hubs: Mutex<HashMap<String, broadcast::Sender<Msg>>>,
    pub presence: Mutex<HashMap<String, HashMap<String, Presence>>>,
    pub direct: Mutex<HashMap<String, Vec<mpsc::UnboundedSender<Msg>>>>,
    pub dirty: AtomicBool,
    pub quit: tokio::sync::Notify,
    pub store: Store,
    pub settings: Mutex<Settings>,
    pub in_flight: Mutex<HashSet<String>>,
    pub commit_notes: Mutex<Vec<String>>,
    pub last_activity: Mutex<std::time::Instant>,
    pub last_night: Mutex<Option<std::time::Instant>>,
    pub night_running: AtomicBool,
    pub syncing: AtomicBool,
    /// When the app asked who should build (0: not yet).
    pub ai_asked_at: std::sync::atomic::AtomicU64,
}

/// Who builds. `heuristic` is built in; `command` hands a prompt to any
/// program (for example `claude -p`) and reads back a JSON plan.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub architect: String,
    pub command: String,
    #[serde(default)]
    pub night: crate::nightshift::Night,
    /// Whether you have chosen an architect (asked once, before the first build).
    #[serde(default)]
    pub asked: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { architect: "heuristic".into(), command: "claude -p".into(), night: Default::default(), asked: false }
    }
}

/// How far out the versions spread: at distance d from spawn, a house shows
/// the version a fraction d/(d+RING) of the way back through history.
pub const RING: f64 = 6.0;

impl App {
    pub fn new(cfg: Config, uni: Universe, store: Store) -> Arc<Self> {
        let settings = std::fs::read(store.root.join(".branches/settings.json")).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default();
        Arc::new(App {
            cfg,
            uni: Mutex::new(uni),
            hubs: Mutex::new(HashMap::new()),
            presence: Mutex::new(HashMap::new()),
            direct: Mutex::new(HashMap::new()),
            dirty: AtomicBool::new(false),
            quit: tokio::sync::Notify::new(),
            store,
            settings: Mutex::new(settings),
            in_flight: Mutex::new(HashSet::new()),
            commit_notes: Mutex::new(Vec::new()),
            last_activity: Mutex::new(std::time::Instant::now()),
            last_night: Mutex::new(None),
            night_running: AtomicBool::new(false),
            syncing: AtomicBool::new(false),
            ai_asked_at: std::sync::atomic::AtomicU64::new(0),
        })
    }

    pub fn save_settings(&self, s: Settings) {
        let _ = std::fs::write(self.store.root.join(".branches/settings.json"), serde_json::to_vec_pretty(&s).unwrap_or_default());
        *self.settings.lock().unwrap() = s;
    }

    /// Quick sketches always use the built-in architect; the configured one
    /// (perhaps a model) gets the builds people are lingering for.
    pub fn architect_for(&self, budget: Budget) -> Arc<dyn Architect> {
        let s = self.settings.lock().unwrap().clone();
        if budget == Budget::Sketch || s.architect != "command" || s.command.trim().is_empty() {
            Arc::new(architect::Heuristic)
        } else {
            Arc::new(architect::Command { cmd: s.command, dir: crate::agent::workdir(&self.store.root) })
        }
    }

    pub fn note_commit(&self, msg: String) {
        self.commit_notes.lock().unwrap().push(msg);
    }

    /// Which version band a house sits in, and that version's tag
    /// (band 0 is the present).
    pub fn band(&self, w: &World, x: i32, z: i32) -> (usize, Option<String>) {
        let n = w.versions.len() + 1;
        if n == 1 {
            return (0, None);
        }
        let spawn = parse_addr(&w.manifest.spawn).unwrap_or((0, 0));
        let d = (x - spawn.0).abs().max((z - spawn.1).abs()) as f64;
        let i = ((n as f64) * d / (d + RING)).floor() as usize;
        let i = i.min(n - 1);
        if i == 0 { (0, None) } else { (i, Some(w.versions[i - 1].tag.clone())) }
    }

    /// The house as people see it at its distance: the present, unless it
    /// sits in an older band and has not been brought forward.
    pub fn displayed(&self, uni: &Universe, world: &str, x: i32, z: i32) -> Option<(Room, Option<String>)> {
        let w = uni.worlds.get(world)?;
        let current = uni.room(world, x, z)?;
        let (_, tag) = self.band(w, x, z);
        match tag {
            Some(t) if !current.replunged => {
                let old = self.store.room_at(&t, world, &addr(x, z)).unwrap_or_else(|| procgen::default_room(&w.manifest, &uni.others(world), x, z));
                Some((old, Some(t)))
            }
            _ => Some((current, None)),
        }
    }

    /// Someone built on a house seen at an older version: that version
    /// becomes its present, keeping the visitor book and what was learned.
    pub fn replunge(&self, uni: &mut Universe, world: &str, x: i32, z: i32) -> Option<String> {
        let (old, tag) = self.displayed(uni, world, x, z)?;
        let tag = tag?;
        let r = uni.room_mut(world, x, z)?;
        let keep = r.clone();
        *r = old;
        r.log = keep.log;
        r.attention = keep.attention;
        r.investors = keep.investors;
        r.weights = keep.weights;
        r.looks = keep.looks;
        r.touches = keep.touches;
        r.building = keep.building;
        r.last_visit = keep.last_visit;
        r.replunged = true;
        Some(tag)
    }

    pub fn hub(&self, world: &str) -> broadcast::Sender<Msg> {
        self.hubs
            .lock()
            .unwrap()
            .entry(world.to_string())
            .or_insert_with(|| broadcast::channel(256).0)
            .clone()
    }

    pub fn broadcast(&self, world: &str, msg: &Value) {
        let _ = self.hub(world).send(Arc::from(msg.to_string()));
    }

    pub fn send_direct(&self, player: &str, msg: &Value) {
        let text: Msg = Arc::from(msg.to_string());
        if let Some(conns) = self.direct.lock().unwrap().get_mut(player) {
            conns.retain(|tx| tx.send(text.clone()).is_ok());
        }
    }

    pub fn touch(&self) {
        self.dirty.store(true, Ordering::Relaxed);
    }

    pub fn threshold(&self, growth: u32) -> f64 {
        self.cfg.pace * (1.0 + 0.6 * growth as f64)
    }
}

impl Universe {
    pub fn seed_builtins(&mut self) {
        for m in procgen::builtin_worlds() {
            // Worlds evolve once they exist; only create missing ones.
            self.worlds.entry(m.id.clone()).or_insert_with(|| World::new(m));
        }
        for w in self.worlds.values_mut() {
            crate::procgen::liminal(&mut w.manifest.generator.params);
        }
    }

    pub fn others(&self, world: &str) -> Vec<WorldManifest> {
        self.worlds.values().filter(|w| w.manifest.id != world).map(|w| w.manifest.clone()).collect()
    }

    /// Stored room or the deterministic default.
    pub fn room(&self, world: &str, x: i32, z: i32) -> Option<Room> {
        let w = self.worlds.get(world)?;
        Some(match w.rooms.get(&addr(x, z)) {
            Some(r) => r.clone(),
            None => procgen::default_room(&w.manifest, &self.others(world), x, z),
        })
    }

    /// Mutable room, materialising the default on first write.
    pub fn room_mut(&mut self, world: &str, x: i32, z: i32) -> Option<&mut Room> {
        let others = self.others(world);
        let w = self.worlds.get_mut(world)?;
        let manifest = &w.manifest;
        Some(w.rooms.entry(addr(x, z)).or_insert_with(|| procgen::default_room(manifest, &others, x, z)))
    }

    pub fn auth(&self, pid: &str, secret: &str) -> Option<&Player> {
        self.players.get(pid).filter(|p| p.secret == secret)
    }
}

/// Compact view used when streaming neighbourhoods.
pub fn room_summary(r: &Room) -> Value {
    json!({
        "x": r.x, "z": r.z, "seed": r.seed, "theme": r.theme,
        "growth": r.growth(),
        "claimed": r.claim.as_ref().map(|c| c.title.clone()),
        "portals": r.portals.len(),
        "building": r.building.is_some(),
    })
}

/// Full public view of a house: its shape as seen at this distance (maybe
/// an older version), with the present visitor book, attention and builds.
/// Investor ids and secrets never leave the host.
pub fn room_view(app: &App, uni: &Universe, world: &str, x: i32, z: i32) -> Option<Value> {
    let (shown, version) = app.displayed(uni, world, x, z)?;
    let r = uni.room(world, x, z)?;
    let w = uni.worlds.get(world)?;
    let mut investors: Vec<(&String, &f64)> = r.investors.iter().collect();
    investors.sort_by(|a, b| b.1.total_cmp(a.1));
    let top: Vec<Value> = investors
        .iter()
        .take(8)
        .map(|(pid, secs)| {
            let name = uni.players.get(*pid).map(|p| p.name.clone()).unwrap_or_else(|| "someone".into());
            json!({ "name": name, "seconds": secs.round() })
        })
        .collect();
    let budget = Budget::from_pace(w.pace);
    Some(json!({
        "world": world,
        "x": x, "z": z, "seed": shown.seed, "theme": shown.theme,
        "theme_name": procgen::theme(&shown.theme).map(|t| t.name).unwrap_or("Unknown"),
        "chambers": shown.chambers,
        "features": shown.features,
        "things": shown.things,
        "portals": shown.portals,
        "version": version,
        "band": app.band(w, x, z).0,
        "log": r.log.iter().rev().take(60).collect::<Vec<_>>(),
        "claim": r.claim,
        "growth": shown.growth(),
        "attention": r.attention.round(),
        "next_growth_at": (app.threshold(shown.growth()) * budget.cost()).round(),
        "budget": budget.name(),
        "building": r.building,
        "leaning": architect::leaning(&r, uni, 3),
        "investors": top,
    }))
}

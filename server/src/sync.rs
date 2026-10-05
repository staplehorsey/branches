//! Bringing worlds in from elsewhere: your own fork (on a new Mac, or after
//! a reinstall) or the main world's `worlds` branch upstream.
//!
//! Fetch, then merge into the local worlds repository. Where both sides
//! changed the same house, your agent (the architect command, such as
//! `claude -p`) is asked to merge the two versions. Anything it can't do
//! falls back to a built-in JSON merge that keeps what both sides added:
//! visitor-book entries, things and doors are joined by id, counters keep
//! the larger value, and on a true tie your side wins.
//!
//! The slow part (the agent) runs without holding the game's lock: it
//! merges against a snapshot commit, then a quick second merge folds in
//! whatever happened while it worked.

use crate::model::Room;
use crate::state::App;
use git2::{Oid, Repository, Signature, build::CheckoutBuilder};
use serde::Serialize;
use serde_json::Value;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

/// Houses one sync may hand to the agent, and how long it may spend.
const MAX_AGENT_FILES: usize = 12;
const AGENT_BUDGET: Duration = Duration::from_secs(15 * 60);
const AGENT_TIMEOUT: Duration = Duration::from_secs(180);

#[derive(Debug, Default, Clone, Serialize)]
pub struct Report {
    pub source: String,
    pub up_to_date: bool,
    /// The source has no worlds branch yet.
    pub nothing_there: bool,
    pub fast_forward: bool,
    /// Files the merge changed on this Mac.
    pub files: usize,
    /// Files both sides had changed.
    pub conflicts: usize,
    /// Of those, merged by the agent.
    pub by_agent: usize,
    /// Worlds that arrived with the merge.
    pub new_worlds: Vec<String>,
    /// Your fork's copy of the code, brought up to date (upstream syncs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fork_code: Option<String>,
}

fn e2s<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

/// Fetch one branch (and optionally its tags) from `url` into a private ref.
/// None when the branch doesn't exist there yet.
pub fn fetch(root: &Path, url: &str, branch: &str, token: Option<&str>, tags: bool) -> Result<Option<Oid>, String> {
    let repo = Repository::open(root).map_err(e2s)?;
    let slug: String = url.trim_end_matches(".git").rsplit(['/', ':']).take(2).collect::<Vec<_>>().join("-");
    let local = format!("refs/branches-sync/{slug}/heads/{branch}");
    if let Ok(mut r) = repo.find_reference(&local) {
        let _ = r.delete();
    }
    let mut remote = repo.remote_anonymous(url).map_err(e2s)?;
    let mut cb = git2::RemoteCallbacks::new();
    if let Some(t) = token {
        let t = t.to_string();
        cb.credentials(move |_, _, _| git2::Cred::userpass_plaintext("x-access-token", &t));
    }
    let mut po = git2::ProxyOptions::new();
    po.auto();
    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(cb);
    fo.proxy_options(po);
    fo.download_tags(git2::AutotagOption::None);
    let mut specs = vec![format!("+refs/heads/{branch}:{local}")];
    if tags {
        specs.push(format!("+refs/tags/*:refs/branches-sync/{slug}/tags/*"));
    }
    if let Err(e) = remote.fetch(&specs, Some(&mut fo), None) {
        if e.message().contains("couldn't find remote ref") || e.code() == git2::ErrorCode::NotFound {
            return Ok(None);
        }
        return Err(format!("could not fetch {url}: {}", e.message()));
    }
    if tags {
        // Version tags this Mac doesn't have yet (never overwrite our own).
        let prefix = format!("refs/branches-sync/{slug}/tags/");
        let mut found: Vec<(String, Oid)> = vec![];
        for r in repo.references_glob(&format!("{prefix}*")).map_err(e2s)? {
            let Ok(r) = r else { continue };
            if let (Some(name), Some(oid)) = (r.name().ok().and_then(|n| n.strip_prefix(prefix.as_str())), r.target()) {
                found.push((name.to_string(), oid));
            }
        }
        for (name, oid) in found {
            let _ = repo.reference(&format!("refs/tags/{name}"), oid, false, "from your fork");
        }
    }
    Ok(repo.refname_to_id(&local).ok())
}

enum Outcome {
    UpToDate,
    FastForward,
    Merged(Oid),
}

/// Work out the merge of `theirs` into `ours` without touching any branch.
fn plan(repo: &Repository, ours: Oid, theirs: Oid, agent: Option<&str>, rep: &mut Report) -> Result<Outcome, String> {
    if ours == theirs || repo.graph_descendant_of(ours, theirs).map_err(e2s)? {
        return Ok(Outcome::UpToDate);
    }
    if repo.graph_descendant_of(theirs, ours).map_err(e2s)? {
        return Ok(Outcome::FastForward);
    }
    let ours_tree = repo.find_commit(ours).and_then(|c| c.tree()).map_err(e2s)?;
    let their_tree = repo.find_commit(theirs).and_then(|c| c.tree()).map_err(e2s)?;
    // Repositories started on different Macs share no history: merge as if
    // from nothing, which the JSON merge handles well.
    let base_tree = match repo.merge_base(ours, theirs) {
        Ok(b) => repo.find_commit(b).and_then(|c| c.tree()).map_err(e2s)?,
        Err(_) => {
            let empty = repo.treebuilder(None).and_then(|t| t.write()).map_err(e2s)?;
            repo.find_tree(empty).map_err(e2s)?
        }
    };
    let mut index = repo.merge_trees(&base_tree, &ours_tree, &their_tree, None).map_err(e2s)?;
    let conflicts: Vec<git2::IndexConflict> = index.conflicts().map_err(e2s)?.flatten().collect();
    rep.conflicts += conflicts.len();
    let started = Instant::now();
    for c in conflicts {
        let read = |e: &Option<git2::IndexEntry>| e.as_ref().and_then(|e| repo.find_blob(e.id).ok()).map(|b| b.content().to_vec());
        let (base, mine, other) = (read(&c.ancestor), read(&c.our), read(&c.their));
        let git2::IndexConflict { ancestor, our, their } = c;
        let Some(template) = our.or(their).or(ancestor) else { continue };
        let path = String::from_utf8_lossy(&template.path).to_string();
        let merged = match (mine, other) {
            (Some(o), Some(t)) => {
                let use_agent = if rep.by_agent < MAX_AGENT_FILES && started.elapsed() < AGENT_BUDGET { agent } else { None };
                Some(resolve(&path, base.as_deref(), &o, &t, use_agent, rep))
            }
            (Some(o), None) => Some(o),
            (None, Some(t)) => Some(t),
            (None, None) => None,
        };
        index.conflict_remove(Path::new(&path)).map_err(e2s)?;
        if let Some(bytes) = merged {
            let id = repo.blob(&bytes).map_err(e2s)?;
            let flags = template.path.len().min(0xfff) as u16;
            let entry = git2::IndexEntry { id, file_size: bytes.len() as u32, flags, flags_extended: 0, ..template };
            index.add(&entry).map_err(e2s)?;
        }
    }
    Ok(Outcome::Merged(index.write_tree_to(repo).map_err(e2s)?))
}

fn pretty(v: &Value) -> Vec<u8> {
    serde_json::to_vec_pretty(v).unwrap_or_default()
}

/// Both sides changed one file: agent first (houses only), then built in.
fn resolve(path: &str, base: Option<&[u8]>, ours: &[u8], theirs: &[u8], agent: Option<&str>, rep: &mut Report) -> Vec<u8> {
    let parse = |b: &[u8]| serde_json::from_slice::<Value>(b).ok();
    let (Some(o), Some(t)) = (parse(ours), parse(theirs)) else { return ours.to_vec() };
    if path.contains("/rooms/") {
        if let Some(cmd) = agent {
            if let Some(v) = ask_agent(cmd, path, base.and_then(parse), &o, &t) {
                rep.by_agent += 1;
                return pretty(&v);
            }
        }
    }
    pretty(&merge_json(&o, &t))
}

pub fn agent_prompt(path: &str, base: Option<&Value>, ours: &Value, theirs: &Value) -> String {
    let show = |v: &Value| serde_json::to_string(v).unwrap_or_default();
    format!(
        "You are merging two versions of one house in Branches, a world of houses that grow.\n\
         File: {path}\n\n\
         MINE is this Mac's copy. THEIRS came from elsewhere (the main world, or another Mac).\n\
         {base}\n\
         MINE:\n{mine}\n\n\
         THEIRS:\n{theirs}\n\n\
         Merge them into one house that keeps what both sides grew: every visitor-book entry \
         (by id, oldest first), every thing and door (by id), the chambers of whichever side grew \
         more, with the other side's distinct chambers appended when they fit. Keep MINE's claim, \
         title and theme when both set them. Keep the same JSON shape and field names.\n\
         Reply with only the merged JSON object.",
        base = base.map(|b| format!("BASE (what both started from):\n{}\n", show(b))).unwrap_or_default(),
        mine = show(ours),
        theirs = show(theirs),
    )
}

fn ask_agent(cmd: &str, path: &str, base: Option<Value>, ours: &Value, theirs: &Value) -> Option<Value> {
    let out = crate::agent::run(cmd, &agent_prompt(path, base.as_ref(), ours, theirs), None, AGENT_TIMEOUT).ok()?;
    let v = crate::architect::extract_json(&out)?;
    // Only accept something that is still a house, at the same address.
    let room: Room = serde_json::from_value(v.clone()).ok()?;
    let mine: Room = serde_json::from_value(ours.clone()).ok()?;
    (room.x == mine.x && room.z == mine.z && !room.chambers.is_empty()).then_some(v)
}

/// The built-in merge. Objects merge key by key; lists of things with an
/// `id` (or a version `tag`) are joined; other lists keep the longer side;
/// numbers keep the larger; anything else keeps ours.
pub fn merge_json(o: &Value, t: &Value) -> Value {
    match (o, t) {
        (Value::Object(a), Value::Object(b)) => {
            let mut m = a.clone();
            for (k, tv) in b {
                let v = match a.get(k) {
                    Some(ov) => merge_json(ov, tv),
                    None => tv.clone(),
                };
                m.insert(k.clone(), v);
            }
            Value::Object(m)
        }
        (Value::Array(a), Value::Array(b)) => {
            let key = |v: &Value| v.get("id").or_else(|| v.get("tag")).map(|k| k.to_string());
            let keyed = !(a.is_empty() && b.is_empty()) && a.iter().chain(b).all(|v| key(v).is_some());
            if !keyed {
                return if b.len() > a.len() { t.clone() } else { o.clone() };
            }
            let mut out = a.clone();
            for tv in b {
                match out.iter_mut().find(|ov| key(ov) == key(tv)) {
                    Some(ov) => *ov = merge_json(ov, tv),
                    None => out.push(tv.clone()),
                }
            }
            // Keep time order, in whichever direction ours ran.
            let at = |v: &Value| v.get("at").and_then(Value::as_f64);
            if out.iter().all(|v| at(v).is_some()) {
                let newest_first = a.len() > 1 && at(&a[0]) > at(&a[a.len() - 1]);
                out.sort_by(|x, y| {
                    let c = at(x).partial_cmp(&at(y)).unwrap_or(std::cmp::Ordering::Equal);
                    if newest_first { c.reverse() } else { c }
                });
            }
            Value::Array(out)
        }
        (Value::Number(a), Value::Number(b)) if b.as_f64() > a.as_f64() => t.clone(),
        _ => o.clone(),
    }
}

/// Make the merge real: a commit on HEAD (or a fast-forward) and a checkout.
fn apply(repo: &Repository, ours: Oid, theirs: Oid, outcome: Outcome, message: &str) -> Result<(), String> {
    let target = match outcome {
        Outcome::UpToDate => return Ok(()),
        Outcome::FastForward => theirs,
        Outcome::Merged(tree) => {
            let tree = repo.find_tree(tree).map_err(e2s)?;
            let sig = Signature::now("Branches architect", "architect@branches.local").map_err(e2s)?;
            let (a, b) = (repo.find_commit(ours).map_err(e2s)?, repo.find_commit(theirs).map_err(e2s)?);
            repo.commit(None, &sig, &sig, message, &tree, &[&a, &b]).map_err(e2s)?
        }
    };
    let mut head = repo.head().map_err(e2s)?.resolve().map_err(e2s)?;
    head.set_target(target, message).map_err(e2s)?;
    repo.checkout_head(Some(CheckoutBuilder::new().force())).map_err(e2s)
}

fn changed(repo: &Repository, from: Oid, to: Oid) -> usize {
    let tree = |o: Oid| repo.find_commit(o).and_then(|c| c.tree()).ok();
    match (tree(from), tree(to)) {
        (Some(a), Some(b)) => repo.diff_tree_to_tree(Some(&a), Some(&b), None).map(|d| d.deltas().len()).unwrap_or(0),
        _ => 0,
    }
}

/// Where to sync from.
pub enum Source {
    /// The main world: `worlds` branch of the upstream repository.
    Upstream,
    /// Your own fork's `worlds` branch (on a new Mac, say).
    Fork,
}

/// Fetch and merge, then reload the worlds the game is playing.
pub fn sync(app: &App, source: Source) -> Result<Report, String> {
    let root = app.store.root.clone();
    let account = crate::github::load(&root);
    let (url, label, token, tags) = match source {
        // BRANCHES_UPSTREAM_URL points at another main world (or a local
        // repository, for testing).
        Source::Upstream => (std::env::var("BRANCHES_UPSTREAM_URL").unwrap_or_else(|_| format!("https://github.com/{}.git", crate::github::UPSTREAM)), format!("the main world ({})", crate::github::UPSTREAM), account.as_ref().map(|a| a.token.clone()), false),
        Source::Fork => {
            let a = account.as_ref().ok_or("connect GitHub first")?;
            let fork = a.fork.clone().ok_or("no fork yet")?;
            (format!("https://github.com/{fork}.git"), format!("your fork ({fork})"), Some(a.token.clone()), true)
        }
    };
    sync_from(app, &url, "worlds", token.as_deref(), tags, &label)
}

pub fn sync_from(app: &App, url: &str, branch: &str, token: Option<&str>, tags: bool, label: &str) -> Result<Report, String> {
    let root = app.store.root.clone();
    let mut rep = Report { source: label.to_string(), ..Default::default() };
    let Some(theirs) = fetch(&root, url, branch, token, tags)? else {
        rep.up_to_date = true;
        rep.nothing_there = true;
        return Ok(rep);
    };
    let agent = {
        let s = app.settings.lock().unwrap();
        (s.architect == "command" && !s.command.trim().is_empty()).then(|| s.command.clone())
    };
    let message = |rep: &Report| {
        let mut m = format!("Merge {label}");
        if rep.conflicts > 0 {
            m += &format!("\n\n{} houses changed on both sides; {} merged by your agent, the rest by Branches.", rep.conflicts, rep.by_agent);
        }
        m
    };
    // 1. Snapshot what's here, then merge against it with nothing locked.
    let snapshot = {
        let uni = app.uni.lock().unwrap();
        app.store.save(&uni).map_err(e2s)?;
        app.store.commit("Before a merge").map_err(e2s)?;
        let repo = Repository::open(&root).map_err(e2s)?;
        repo.head().and_then(|h| h.peel_to_commit()).map(|c| c.id()).map_err(e2s)?
    };
    let repo = Repository::open(&root).map_err(e2s)?;
    let first = match plan(&repo, snapshot, theirs, agent.as_deref(), &mut rep)? {
        Outcome::UpToDate => {
            rep.up_to_date = true;
            return Ok(rep);
        }
        Outcome::FastForward => theirs,
        Outcome::Merged(tree) => {
            let tree = repo.find_tree(tree).map_err(e2s)?;
            let sig = Signature::now("Branches architect", "architect@branches.local").map_err(e2s)?;
            let (a, b) = (repo.find_commit(snapshot).map_err(e2s)?, repo.find_commit(theirs).map_err(e2s)?);
            repo.commit(None, &sig, &sig, &message(&rep), &tree, &[&a, &b]).map_err(e2s)?
        }
    };
    rep.fast_forward = first == theirs;
    // 2. Fold in anything that happened meanwhile, apply, and reload.
    let mut uni = app.uni.lock().unwrap();
    app.store.save(&uni).map_err(e2s)?;
    app.store.commit("Before a merge").map_err(e2s)?;
    let now = repo.head().and_then(|h| h.peel_to_commit()).map(|c| c.id()).map_err(e2s)?;
    let mut quick = Report::default();
    let outcome = plan(&repo, now, first, None, &mut quick)?;
    apply(&repo, now, first, outcome, &message(&rep))?;
    let after = repo.head().and_then(|h| h.peel_to_commit()).map(|c| c.id()).map_err(e2s)?;
    rep.files = changed(&repo, now, after);
    app.store.forget();
    let mut fresh = app.store.load();
    fresh.seed_builtins();
    fresh.players = std::mem::take(&mut uni.players);
    rep.new_worlds = fresh.worlds.keys().filter(|k| !uni.worlds.contains_key(*k)).cloned().collect();
    *uni = fresh;
    app.dirty.store(true, Ordering::Relaxed);
    Ok(rep)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn json_merge_keeps_both_sides() {
        let ours = json!({ "x": 1, "z": 2, "attention": 10.0, "log": [{ "id": "a", "at": 1 }, { "id": "c", "at": 3 }], "chambers": [1, 2], "claim": { "title": "mine" } });
        let theirs = json!({ "x": 1, "z": 2, "attention": 30.0, "log": [{ "id": "a", "at": 1 }, { "id": "b", "at": 2 }], "chambers": [1, 2, 3], "claim": { "title": "theirs" }, "things": [{ "id": "t" }] });
        let m = merge_json(&ours, &theirs);
        assert_eq!(m["attention"], 30.0);
        assert_eq!(m["log"].as_array().unwrap().iter().map(|e| e["id"].as_str().unwrap()).collect::<String>(), "abc");
        assert_eq!(m["chambers"].as_array().unwrap().len(), 3);
        assert_eq!(m["claim"]["title"], "mine");
        assert_eq!(m["things"][0]["id"], "t");
        // Versions run newest first and stay that way.
        let v = merge_json(&json!([{ "tag": "v2", "at": 2 }, { "tag": "v1", "at": 1 }]), &json!([{ "tag": "v3", "at": 3 }, { "tag": "v1", "at": 1 }]));
        assert_eq!(v.as_array().unwrap().iter().map(|e| e["tag"].as_str().unwrap()).collect::<String>(), "v3v2v1");
    }
}

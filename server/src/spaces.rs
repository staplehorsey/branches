//! Spaces: beyond a house's entry hall, the dream is made of spaces that
//! Claude Code writes from scratch, each a self-contained web page in its
//! own folder (three.js, a canvas, shaders, sound, words: anything). The
//! game only moves you in, onward and back out; nothing else is imposed.
//!
//! They are made ahead of you: entering a house starts its first space,
//! entering space n starts space n+1. Each space is asked to hold a curious
//! person for longer than the next one takes to make, so that by the time
//! you reach for something new it is already there.
//!
//! Files live with the worlds (worlds/<world>/spaces/<x>_<z>/<n>/) and are
//! served at /space/<world>/<x>,<z>/<n>/, with a small bridge script
//! injected so the reserved keys and the way onward always work. Spaces run
//! in a sandboxed frame with no access to the app.

use crate::model::*;
use crate::state::App;
use axum::extract::{Path as UrlPath, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Space {
    pub n: u32,
    /// building, ready or failed.
    pub status: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub summary: String,
    /// How long it was designed to hold someone.
    #[serde(default)]
    pub minutes: f32,
    pub started: u64,
    /// Seconds it took to make.
    #[serde(default)]
    pub secs: u64,
    #[serde(default)]
    pub error: Option<String>,
    /// Seconds people have spent in it, and how many visits.
    #[serde(default)]
    pub dwell: f64,
    #[serde(default)]
    pub visits: u32,
    #[serde(default)]
    pub tries: u32,
}

/// Spaces being made at once.
const MAX_BUILDS: usize = 2;
const BUILD_TIMEOUT: Duration = Duration::from_secs(20 * 60);
/// Keep making spaces for a house only while someone was there lately.
const RECENT_MS: u64 = 30 * 60_000;

/// Spaces replace grown chambers when Claude Code is the architect in the app.
pub fn enabled(app: &App) -> bool {
    let s = app.settings.lock().unwrap();
    app.cfg.app && s.architect == "command" && s.command.split_whitespace().next().is_some_and(|w| w == "claude" || w.ends_with("/claude")) && crate::agent::find_claude().is_some()
}

fn dir(root: &Path, world: &str, x: i32, z: i32, n: u32) -> PathBuf {
    root.join("worlds").join(world).join("spaces").join(format!("{x}_{z}")).join(n.to_string())
}

/// Seconds a space typically takes to make: the median of the latest ones.
fn typical_secs(uni: &Universe) -> u64 {
    let mut done: Vec<(u64, u64)> = uni.worlds.values().flat_map(|w| w.rooms.values()).flat_map(|r| r.spaces.iter()).filter(|s| s.status == "ready" && s.secs > 0).map(|s| (s.started, s.secs)).collect();
    done.sort();
    let mut last: Vec<u64> = done.iter().rev().take(7).map(|d| d.1).collect();
    if last.is_empty() {
        return 180;
    }
    last.sort();
    last[last.len() / 2]
}

/// Someone entered space `n` of a house (0 is the entry hall), maybe
/// leaving space `left` after `seconds`. Make sure the next one is coming.
pub fn visit(app: &Arc<App>, world: &str, x: i32, z: i32, n: u32, left: Option<(u32, f64)>) -> Result<Value, String> {
    {
        let mut uni = app.uni.lock().unwrap();
        let r = uni.room_mut(world, x, z).ok_or("no house there")?;
        if let Some((m, secs)) = left {
            if let Some(s) = r.spaces.iter_mut().find(|s| s.n == m) {
                s.dwell += secs.clamp(0.0, 3.0 * 3600.0);
                s.visits += 1;
            }
        }
        r.space_visited = r.space_visited.max(n);
        r.space_seen_at = now_ms();
    }
    app.touch();
    ensure_all(app);
    let uni = app.uni.lock().unwrap();
    let r = uni.room(world, x, z).ok_or("no house there")?;
    Ok(json!({ "spaces": r.spaces, "typical_secs": typical_secs(&uni) }))
}

/// Start the spaces people are about to want, up to the limit.
pub fn ensure_all(app: &Arc<App>) {
    if !enabled(app) {
        return;
    }
    let now = now_ms();
    let mut start = Vec::new();
    {
        let mut uni = app.uni.lock().unwrap();
        let mut building: usize = uni.worlds.values().flat_map(|w| w.rooms.values()).flat_map(|r| r.spaces.iter()).filter(|s| s.status == "building").count();
        let mut wanted: Vec<(u64, String, String)> = Vec::new();
        for (wid, w) in &uni.worlds {
            for (key, r) in &w.rooms {
                if now.saturating_sub(r.space_seen_at) > RECENT_MS || r.spaces.iter().any(|s| s.status == "building") {
                    continue;
                }
                let next = r.space_visited + 1;
                let have = r.spaces.iter().filter(|s| s.status == "ready").count() as u32;
                let stuck = r.spaces.last().is_some_and(|s| s.status == "failed" && s.tries >= 3);
                if have < next && !stuck {
                    wanted.push((r.space_seen_at, wid.clone(), key.clone()));
                }
            }
        }
        // Whoever was there most recently first.
        wanted.sort_by(|a, b| b.0.cmp(&a.0));
        for (_, wid, key) in wanted {
            if building >= MAX_BUILDS {
                break;
            }
            let Some(r) = uni.worlds.get_mut(&wid).and_then(|w| w.rooms.get_mut(&key)) else { continue };
            let n = match r.spaces.last() {
                Some(s) if s.status == "failed" => s.n,
                Some(s) => s.n + 1,
                None => 1,
            };
            let tries = r.spaces.last().filter(|s| s.status == "failed").map(|s| s.tries).unwrap_or(0) + 1;
            r.spaces.retain(|s| s.n != n);
            r.spaces.push(Space { n, status: "building".into(), started: now, tries, ..Default::default() });
            building += 1;
            start.push((wid, r.x, r.z, n));
        }
    }
    for (wid, x, z, n) in start {
        app.broadcast(&wid, &json!({ "t": "space", "x": x, "z": z, "n": n, "status": "building" }));
        let app = app.clone();
        std::thread::spawn(move || build(&app, &wid, x, z, n));
    }
}

/// Spaces left half-made by a restart are made again.
pub fn recover(app: &App) {
    let mut uni = app.uni.lock().unwrap();
    for w in uni.worlds.values_mut() {
        for r in w.rooms.values_mut() {
            for s in r.spaces.iter_mut().filter(|s| s.status == "building") {
                s.status = "failed".into();
                s.error = Some("interrupted".into());
            }
        }
    }
}

fn build(app: &Arc<App>, world: &str, x: i32, z: i32, n: u32) {
    let root = app.store.root.clone();
    let d = dir(&root, world, x, z, n);
    let started = std::time::Instant::now();
    let result = (|| -> Result<Value, String> {
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).map_err(|e| e.to_string())?;
        let prompt = {
            let uni = app.uni.lock().unwrap();
            brief(app, &uni, world, x, z, n)
        };
        let _ = std::fs::write(d.join("BRIEF.md"), &prompt);
        let cmd = {
            let s = app.settings.lock().unwrap();
            format!("{} --permission-mode acceptEdits --allowedTools Read Write Edit Glob Grep", s.command.trim())
        };
        crate::agent::run(&cmd, &prompt, Some(&d), BUILD_TIMEOUT)?;
        if !d.join("index.html").is_file() {
            return Err("it didn't write index.html".into());
        }
        let meta: Value = std::fs::read(d.join("space.json")).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or(json!({}));
        Ok(meta)
    })();
    let secs = started.elapsed().as_secs();
    let (status, name) = {
        let mut uni = app.uni.lock().unwrap();
        let Some(r) = uni.worlds.get_mut(world).and_then(|w| w.rooms.get_mut(&addr(x, z))) else { return };
        let now = now_ms();
        let mut idea_from_space = None;
        let Some(s) = r.spaces.iter_mut().find(|s| s.n == n) else { return };
        s.secs = secs;
        match &result {
            Ok(meta) => {
                s.status = "ready".into();
                s.error = None;
                s.name = meta["name"].as_str().unwrap_or("").chars().take(80).collect();
                s.summary = meta["summary"].as_str().unwrap_or("").chars().take(600).collect();
                s.minutes = meta["minutes"].as_f64().unwrap_or(0.0) as f32;
                if let (Some(name), Some(premise)) = (meta["idea"]["name"].as_str(), meta["idea"]["premise"].as_str()) {
                    idea_from_space = Some((name.chars().take(80).collect::<String>(), premise.chars().take(1200).collect::<String>()));
                }
            }
            Err(e) => {
                tracing::warn!("space {world} {x},{z} #{n} failed: {e}");
                s.status = "failed".into();
                s.error = Some(e.chars().take(300).collect());
            }
        }
        let (status, name) = (s.status.clone(), s.name.clone());
        match (&mut r.idea, idea_from_space) {
            (None, Some((name, premise))) => {
                let muses = crate::dream::load(&root).muses.iter().rev().take(8).map(|m| m.name.clone()).collect();
                r.idea = Some(Idea { name, premise, look: None, muses, refined: 1, at: now });
            }
            (Some(i), Some((_, premise))) => {
                i.premise = premise;
                i.refined += 1;
            }
            (Some(i), None) if status == "ready" => i.refined += 1,
            _ => {}
        }
        (status, name)
    };
    if status == "ready" {
        let wname = app.uni.lock().unwrap().worlds.get(world).map(|w| w.manifest.name.clone()).unwrap_or_default();
        app.note_commit(format!("{wname} {x},{z}: space {n}, {name} [claude, {secs}s]"));
    }
    app.touch();
    app.broadcast(world, &json!({ "t": "space", "x": x, "z": z, "n": n, "status": status, "name": name }));
    ensure_all(app);
}

const BRIDGE_KEYS: &str = "Esc (back to the house), = and - (warmer, colder), G (feedback), ? (keys), ` (the workshop)";

fn brief(app: &App, uni: &Universe, world: &str, x: i32, z: i32, n: u32) -> String {
    let Some(r) = uni.room(world, x, z) else { return String::new() };
    let wname = uni.worlds.get(world).map(|w| w.manifest.name.clone()).unwrap_or_default();
    let typical = typical_secs(uni);
    let hold = (typical as f64 * 1.5 / 60.0).ceil().max(2.0);
    let idea = match &r.idea {
        Some(i) => format!("This house's idea: {} — {}\n(Refined {} times so far. Take it one step further: more specific, stranger, more itself. Never change the subject; do transform the form.)", i.name, i.premise, i.refined),
        None => "This house has no idea yet: give it one (in space.json, \"idea\"). An idea is a feeling or theme with a point of view, not a room type: \"the difficult emotions of futility, desperation and love in the song Jolene\", \"vast empty sand where light is the only thing to pay attention to\", \"The Brothers Karamazov: faith and doubt at one long kitchen table\". Draw it from the dreamer's muses and compass, as a variation on the neighbours.".into(),
    };
    let mut before = String::new();
    for s in r.spaces.iter().filter(|s| s.status == "ready" && s.n < n) {
        let stayed = if s.visits > 0 { format!("people stayed {:.0}s on average over {} visits", s.dwell / s.visits as f64, s.visits) } else { "not visited yet".into() };
        before.push_str(&format!("- space {} ({}/): {} — {} (designed for {:.0} min; {stayed})\n", s.n, s.n, s.name, s.summary, s.minutes));
    }
    let marks: Vec<String> = crate::dream::load(&app.store.root).compass.iter().rev().filter(|m| m.world == world && m.x == x && m.z == z).take(8).map(|m| format!("- {} on {}{}", if m.score > 0 { "warmer" } else if m.score < 0 { "colder" } else { "a note" }, m.idea, m.note.as_ref().map(|t| format!(": \"{t}\"")).unwrap_or_default())).collect();
    format!(
        r#"You are making space {n} of a house in Branches: a dream someone wanders through. Endless streets of identical liminal houses; inside each, beyond its entry hall, a chain of spaces you make, each continuing the last like a dream continues.

{idea}

The house is at {x},{z} in the world {wname}.
{before_title}{before}{marks_title}{marks}
{dream}
WHAT TO MAKE
A complete, self-contained web page in THIS folder: index.html plus any other files you want (scripts, shaders, generated textures, sounds made with WebAudio). There are no required building blocks and no fixed visual language: a three.js world, a 2D canvas, a text adventure, a shader you drift through, a field of sound, an impossible architecture. Make the medium serve the idea. It should feel like the next moment of a dream: continue the previous space, then transform it.

TIME
The next space takes about {typical}s to make, and starts being made the moment this one is entered. Design this one to hold a curious person for at least {hold} minutes before the way onward can be taken: a maze solvable in about that time, a slow reveal, a puzzle, a long walk toward something, a thing that unfolds as they pay attention. The way onward should be found, not given, and should not appear before then.

THE CONTRACT (the only rules)
- It runs full screen in a sandboxed iframe (scripts and pointer lock allowed; no access to the app, cookies or storage beyond memory).
- To go onward, call branches.onward() (or parent.postMessage({{branches: "onward"}}, "*")). To go back to the house, branches.back().
- branches.say("text") shows a line of text over the space.
- These keys are reserved by the game; don't use them: {keys}. Esc must never be trapped.
- Use only files in this folder and this server: three.js is at /vendor/three.module.js (import * as THREE from "/vendor/three.module.js"); /kit/space.js is an optional first-person helper (read it at {kit} if you want it). No other network.
- Write space.json: {{"name": "a name", "summary": "2-3 sentences: what this space is, how it continues the idea, how the way onward is found", "minutes": how long it should hold someone{idea_json}}}.
- Test your logic by reading it carefully; you can't open a browser. Keep it robust: a blank screen is the worst outcome.
- Previous spaces are in ../<n>/ if you want to read them.
"#,
        before_title = if before.is_empty() { "" } else { "Spaces before this one in this house:\n" },
        marks_title = if marks.is_empty() { "" } else { "What the dreamer said here:\n" },
        marks = marks.join("\n") + if marks.is_empty() { "" } else { "\n" },
        dream = crate::dream::guidance(&app.store.root, uni, world, x, z),
        keys = BRIDGE_KEYS,
        kit = app.cfg.client_dir.join("kit/space.js").canonicalize().map(|p| p.display().to_string()).unwrap_or_else(|_| "client/kit/space.js".into()),
        idea_json = if r.idea.is_none() { ", \"idea\": {\"name\": \"...\", \"premise\": \"2-4 sentences\"}" } else { ", \"idea\": {\"name\": \"(same)\", \"premise\": \"what the idea has become\"} (optional)" },
    )
}

// ------------------------------------------------------------------ serving

/// GET /space/{world}/{x,z}/{n}/{*path}: a space's files. HTML pages get
/// the bridge, so the reserved keys and the way onward always work.
pub async fn serve(State(app): State<Arc<App>>, UrlPath((world, at, n, path)): UrlPath<(String, String, u32, String)>) -> Response {
    let Some((x, z)) = parse_addr(&at) else { return StatusCode::NOT_FOUND.into_response() };
    let path = if path.is_empty() || path.ends_with('/') { format!("{path}index.html") } else { path };
    if path.split('/').any(|p| p == ".." || p.starts_with('.')) || world.contains("..") || world.contains('/') {
        return StatusCode::NOT_FOUND.into_response();
    }
    let file = dir(&app.store.root, &world, x, z, n).join(&path);
    let Ok(bytes) = tokio::fs::read(&file).await else { return StatusCode::NOT_FOUND.into_response() };
    let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let ctype = match ext.as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "wav" => "audio/wav",
        "mp3" => "audio/mpeg",
        "glsl" | "txt" | "frag" | "vert" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    };
    let body = if ext == "html" || ext == "htm" {
        let html = String::from_utf8_lossy(&bytes);
        let bridge = r#"<script src="/kit/bridge.js"></script>"#;
        match html.find("<head>").or_else(|| html.find("<head ")).and_then(|i| html[i..].find('>').map(|j| i + j + 1)) {
            Some(i) => format!("{}{bridge}{}", &html[..i], &html[i..]).into_bytes(),
            None => format!("{bridge}{html}").into_bytes(),
        }
    } else {
        bytes
    };
    ([(header::CONTENT_TYPE, ctype), (header::CACHE_CONTROL, "no-cache")], body).into_response()
}

pub async fn serve_index(state: State<Arc<App>>, UrlPath((world, at, n)): UrlPath<(String, String, u32)>) -> Response {
    serve(state, UrlPath((world, at, n, String::new()))).await
}

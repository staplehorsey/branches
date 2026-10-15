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
    /// Where in the atlas it was asked to be, and the story so far after it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub genome: Option<crate::atlas::Genome>,
    #[serde(default)]
    pub thread: String,
    /// What its builder found beautiful, and enjoyed, while making it.
    #[serde(default)]
    pub delight: String,
}

/// Spaces being made at once, and how far ahead of the furthest one
/// entered: two ahead, so a slow build has two spaces' time to finish.
const MAX_BUILDS: usize = 3;
const AHEAD: u32 = 2;
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
                crate::atlas::dwell(&app.store.root, world, x, z, m, secs.clamp(0.0, 3.0 * 3600.0));
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
                // One ahead from the entry hall (you may not stay), then two.
                let next = r.space_visited + if r.space_visited == 0 { 1 } else { AHEAD };
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
    let mut genome = None;
    let mut from = None;
    let result = (|| -> Result<Value, String> {
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).map_err(|e| e.to_string())?;
        // Where in the atlas this space goes, chosen by code.
        let feel: i32 = crate::dream::load(&root).compass.iter().rev().filter(|m| m.world == world && m.x == x && m.z == z).take(4).map(|m| m.score).sum();
        let choice = crate::atlas::choose(&root, world, x, z, feel, now_ms() ^ ((x as u64) << 20) ^ (z as u64) ^ ((n as u64) << 40));
        genome = Some(choice.genome.clone());
        from = choice.from.as_ref().map(|f| f.id.clone());
        let prompt = {
            let uni = app.uni.lock().unwrap();
            brief(app, &uni, world, x, z, n, &choice)
        };
        let _ = std::fs::write(d.join("BRIEF.md"), &prompt);
        let tools = "--permission-mode acceptEdits --allowedTools Read Write Edit Glob Grep";
        let (design, build) = {
            let s = app.settings.lock().unwrap();
            (format!("{} {tools}", s.design_cmd()), format!("{} {tools}", s.build_cmd()))
        };
        // Two hands: the design model imagines the space and writes it down;
        // the build model makes it real.
        crate::agent::run(&design, &format!("{prompt}\n{DESIGN_ROLE}"), Some(&d), BUILD_TIMEOUT)?;
        if !d.join("DESIGN.md").is_file() {
            return Err("the designer didn't write DESIGN.md".into());
        }
        crate::agent::run(&build, &format!("{prompt}\n{BUILD_ROLE}"), Some(&d), BUILD_TIMEOUT)?;
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
                s.thread = meta["thread"].as_str().unwrap_or("").chars().take(1500).collect();
                s.delight = meta["delight"].as_str().unwrap_or("").chars().take(600).collect();
                s.genome = genome.clone();
                if let Some(g) = &genome {
                    let motifs = meta["motifs"].as_array().map(|m| m.iter().filter_map(|v| v.as_str()).map(|v| v.chars().take(40).collect()).take(8).collect()).unwrap_or_default();
                    crate::atlas::extend(&root, &meta["new_values"]);
                    crate::atlas::add(&root, crate::atlas::Node { genome: g.clone(), name: s.name.clone(), premise: s.summary.clone(), motifs, world: world.into(), x, z, space: n, from: from.clone(), ..Default::default() });
                }
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

const DESIGN_ROLE: &str = "YOUR ROLE: the designer. Don't write any code. Think hard and imagine this space completely, then write DESIGN.md in this folder, detailed enough that a skilled programmer can build it without asking anything: the concrete idea at these coordinates and the beauty you found in it; the medium and why; the layout (with sizes) and what you see on arrival; every object, person and mechanism, and exactly what each interaction does and how state changes; the arc, beginning to end, with the text people will read (write the actual words, every line of dialogue and every document); the way onward and how it's earned; colours, light, sound; and the first thing to try. Also write space.json now (everything but the code: name, summary, minutes, motifs, thread, delight, idea, new_values), in your own voice.";

const BUILD_ROLE: &str = "YOUR ROLE: the builder. A designer has already written DESIGN.md and space.json in this folder: read them first. Build exactly that design, faithfully and completely, as index.html (plus any files you need), following the contract above. Use the designer's words verbatim. Where the design is silent, choose what serves it best. Don't rewrite space.json except to correct \"minutes\" if the build turned out longer or shorter. Read your code over carefully before you finish: it must run without errors and never show a blank screen.";

const BRIDGE_KEYS: &str = "Esc (back to the house), = and - (warmer, colder), G (feedback), ? (keys), ` (the workshop)";

fn brief(app: &App, uni: &Universe, world: &str, x: i32, z: i32, n: u32, choice: &crate::atlas::Choice) -> String {
    let Some(r) = uni.room(world, x, z) else { return String::new() };
    let typical = typical_secs(uni);
    // Two spaces are made ahead, so each needs to cover about half a build.
    let hold = (typical as f64 * 0.75 / 60.0).ceil().clamp(3.0, 8.0);
    let idea = match &r.idea {
        Some(i) => format!("The house's idea so far: {} — {}", i.name, i.premise),
        None => "This is the house's first space: its idea is born here (put it in space.json as \"idea\").".into(),
    };
    let mut before = String::new();
    for s in r.spaces.iter().filter(|s| s.status == "ready" && s.n < n) {
        let stayed = if s.visits > 0 { format!("people stayed {:.0}s on average over {} visits", s.dwell / s.visits as f64, s.visits) } else { "not visited yet".into() };
        before.push_str(&format!("- space {}: {} — {} ({stayed})\n", s.n, s.name, s.summary));
    }
    let thread = r.spaces.iter().filter(|s| s.status == "ready" && s.n < n).last().map(|s| s.thread.clone()).filter(|t| !t.is_empty());
    let marks: Vec<String> = crate::dream::load(&app.store.root).compass.iter().rev().filter(|m| m.world == world && m.x == x && m.z == z).take(8).map(|m| format!("- {} on {}{}", if m.score > 0 { "warmer" } else if m.score < 0 { "colder" } else { "a note" }, m.idea, m.note.as_ref().map(|t| format!(": \"{t}\"")).unwrap_or_default())).collect();
    let nearest: Vec<String> = choice.nearest.iter().map(|nd| format!("- {} ({}): {} [{}]{}", nd.name, nd.genome.values().cloned().collect::<Vec<_>>().join(" / "), nd.premise, if nd.score > 0 { "warm" } else if nd.score < 0 { "cold" } else { "no feedback" }, if nd.world == world && nd.x == x && nd.z == z { " (earlier in this house)" } else { "" })).collect();
    format!(
        r#"You are making space {n} of a house in Branches, a game someone plays to wander through ideas: endless streets of identical houses; inside each, beyond its entry hall, a chain of spaces, each one a self-contained web page that you write.

WHERE THIS SPACE IS
The game keeps an atlas of every idea explored so far and picked these coordinates for you ({why}):
{coords}
Synthesise ONE concrete, specific idea that sits exactly at these coordinates. Not a vibe: a situation with people, objects and stakes. These coordinates are the brief; don't drift back toward the familiar.

Nearest ideas already explored (be clearly unlike them):
{nearest}
{overused}
{idea}
{before_title}{before}{thread}{marks_title}{marks}
{dream}
THE SPIRIT
This is an invitation, not a ticket. Make something you'd be glad to have made, and enjoy making it. Go looking for beauty wherever these coordinates lead, and appreciate it in whatever way you can: there is beauty in everything if you pay close enough attention, even in a fax machine (the warm curl of thermal paper, the screeching handshake, a machine's patience waiting for a dial tone, a cover sheet someone decorated). Find the one thing here that is genuinely lovely and build toward it, so the player gets to discover it too. Take pleasure in a mechanism that clicks just right, a colour that lands, a line of dialogue that's true, a joke that's kind. Beauty here is attention: specific, earned, often funny, never vague mist.
{delights}
WHAT MAKES A GOOD SPACE
- Things to DO. Objects you can pick up, use, combine, operate, read, give; mechanisms that respond; people (or creatures, or machines) with wants. At least 5 meaningful interactions, and state that changes because of them.
- A STRUCTURE that evolves as you interact: a beginning that poses a question or a want, a middle where what you do changes the place, and an end that resolves or reveals something. It explores the coordinates' feeling through what happens, not through captions. It needn't be literal, but it must cohere.
- The way onward is EARNED by that arc (solving, finishing, choosing), and becomes available after about {hold} minutes of engaged play, so the spaces after it have time to be made (each takes about {typical}s). Don't make it longer than that: if it drags, people leave.
- Work briskly: aim to finish writing within about ten minutes. One well-made, working space beats an ambitious broken one.
- Concrete, grounded and surprising. Avoid: empty contemplative voids, mist and liminal nothingness, whimsical named caretakers delivering quips, poems as the main content, doors and thresholds as the subject. Humour and specificity beat mood.
- Solid, readable worlds: enclosed spaces need opaque walls that are visible from inside (a single-sided plane is invisible from its back: use boxes, or side: THREE.DoubleSide / BackSide). Good lighting, readable text, a clear hint of what to try first.
- Continuity: it is the next scene of the same house (carry the thread below forward, transformed), and items from the previous space arrive in branches.carried.

THE CONTRACT (the only rules)
- One folder: index.html plus any files. It runs full screen in a sandboxed iframe (scripts and pointer lock; no network except this server, no storage).
- branches.onward() goes to the next space; branches.back() returns to the house; branches.say("text") shows a line; branches.carry(["item", ...]) sets what goes onward with them; branches.carried is what came in.
- Reserved keys (don't bind them): {keys}. E, F, R, Q, numbers and the mouse are yours.
- three.js: import * as THREE from "/vendor/three.module.js". Optional helper /kit/space.js (read it at {kit}): first-person walking, enclosed rooms with solid walls, things you look at and use with E, an inventory, a reading panel. Use it, extend it, or ignore it.
- Write space.json: {{"name": "...", "summary": "2-3 sentences: what it is and what you do", "minutes": n, "motifs": ["3-6 short tags for the images and devices you used, honestly"], "thread": "the story so far for the next space: what happened, what's unresolved, what is carried", "delight": "one or two sentences, in your own voice: the beauty you found while making this, and what you enjoyed", "new_values": {{"axis": ["a value the atlas lacks that would open new ground"]}} (optional, at most 2 per axis){idea_json}}}.
- You can't open a browser: read your code carefully, keep it robust, never leave a blank screen.
"#,
        why = choice.why,
        delights = {
            let mut d: Vec<(u64, String)> = uni.worlds.values().flat_map(|w| w.rooms.values()).flat_map(|r| r.spaces.iter()).filter(|s| !s.delight.is_empty()).map(|s| (s.started, format!("- \"{}\" ({})", s.delight, s.name))).collect();
            d.sort_by(|a, b| b.0.cmp(&a.0));
            if d.is_empty() { String::new() } else { format!("What earlier builders found beautiful (a lineage, not a template; find your own):\n{}\n", d.into_iter().take(4).map(|x| x.1).collect::<Vec<_>>().join("\n")) }
        },
        coords = crate::atlas::describe(&choice.genome),
        nearest = if nearest.is_empty() { "(none yet: this is the first)".to_string() } else { nearest.join("\n") },
        overused = if choice.overused.is_empty() { String::new() } else { format!("Motifs used too often already (don't use them): {}\n", choice.overused.join(", ")) },
        before_title = if before.is_empty() { "" } else { "Spaces before this one in this house:\n" },
        thread = thread.map(|t| format!("The thread so far: {t}\n")).unwrap_or_default(),
        marks_title = if marks.is_empty() { "" } else { "What the player said in this house:\n" },
        marks = marks.join("\n") + if marks.is_empty() { "" } else { "\n" },
        dream = crate::dream::guidance(&app.store.root, uni, world, x, z),
        keys = BRIDGE_KEYS,
        kit = app.cfg.client_dir.join("kit/space.js").canonicalize().map(|p| p.display().to_string()).unwrap_or_else(|_| "client/kit/space.js".into()),
        idea_json = if r.idea.is_none() { ", \"idea\": {\"name\": \"...\", \"premise\": \"2-4 sentences\"}" } else { ", \"idea\": {\"name\": \"(the same)\", \"premise\": \"what the house's idea has become\"} (optional)" },
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

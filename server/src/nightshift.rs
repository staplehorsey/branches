//! The night shift: when nobody is around (and only if you turn it on), the
//! app hands a bigger project to your agent (for example Claude Code on a
//! subscription you aren't using overnight) and lets it work for a long
//! while: a whole zoo tycoon, a go-kart track. The result is a self-contained
//! game in the world's repository, opened in-world as an attraction.

use crate::model::*;
use crate::state::App;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Night {
    pub enabled: bool,
    /// Runs in the project folder with the brief on stdin.
    pub command: String,
    /// Local hours, start inclusive, end exclusive (may wrap past midnight).
    pub start_hour: u32,
    pub end_hour: u32,
    /// Minutes with nobody playing before it starts.
    pub idle_minutes: u32,
    /// The most a single project may take.
    pub max_minutes: u32,
}

impl Default for Night {
    fn default() -> Self {
        Night { enabled: false, command: "claude -p --permission-mode acceptEdits".into(), start_hour: 1, end_hour: 7, idle_minutes: 20, max_minutes: 120 }
    }
}

const IDEAS: &[(&str, &str)] = &[
    ("Zoo Tycoon", "A cozy zoo tycoon: lay out paths and enclosures on a grid, choose animals, keep visitors and animals happy, with a small economy, day/night and a goal to reach five stars."),
    ("Go-Kart Dreams", "A go-kart racer: a winding track through this world's scenery, three laps against two rival karts, drifting, a boost pad or two, lap times saved as records."),
    ("Little Train Set", "A tiny train set: lay track pieces on a table, place stations and houses, run trains that pick up passengers on a timetable."),
    ("Aquarium Keeper", "An aquarium keeper: design tanks, balance water, feed and breed gentle fish, decorate with plants and stones, and welcome visitors."),
    ("Lantern Festival", "A lantern festival planner: string lanterns and stalls across a night square so crowds flow pleasantly, scored by delight and wonder."),
    ("Greenhouse", "A greenhouse sim: plant, water, cross-pollinate and sell rare flowers over seasons; unlock new glasshouses."),
    ("Marble Run", "A marble run builder: drag pieces onto a wall, set marbles loose, hit every bell; physics that feels good."),
    ("Night Market", "A night market stall: cook and serve a queue of dreamy customers with recipes that get more complex each night."),
];

const GAME_WORDS: &[&str] = &["game", "tycoon", "race", "racing", "kart", "track", "puzzle", "arcade", "zoo", "maze", "sim", "build a", "play"];

pub struct Project {
    pub title: String,
    pub idea: String,
    pub world: String,
    pub x: i32,
    pub z: i32,
    pub slug: String,
}

fn local_hour() -> u32 {
    std::process::Command::new("date").arg("+%H").output().ok().and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse().ok()).unwrap_or(3)
}

fn in_window(n: &Night, h: u32) -> bool {
    if n.start_hour <= n.end_hour { h >= n.start_hour && h < n.end_hour } else { h >= n.start_hour || h < n.end_hour }
}

/// What to build tonight, and where: a game someone asked for in a visitor
/// book if there is one, else the next idea; in the house people linger in most.
pub fn choose(uni: &Universe, built: &[String]) -> Option<Project> {
    let mut asked: Option<(u64, String, String, i32, i32)> = None;
    for w in uni.worlds.values() {
        for r in w.rooms.values() {
            for l in r.log.iter().filter(|l| l.kind == "request") {
                let t = l.text.to_lowercase();
                if GAME_WORDS.iter().any(|g| t.contains(g)) && !built.iter().any(|b| b == &l.text) && asked.as_ref().is_none_or(|a| l.at > a.0) {
                    asked = Some((l.at, l.text.clone(), w.manifest.id.clone(), r.x, r.z));
                }
            }
        }
    }
    let (title, idea, world, x, z) = match asked {
        Some((_, text, w, x, z)) => {
            let title: String = text.split_whitespace().take(5).collect::<Vec<_>>().join(" ");
            (title, text, w, x, z)
        }
        None => {
            let (t, i) = IDEAS.iter().find(|(t, _)| !built.iter().any(|b| b == t)).or(IDEAS.first())?;
            // The most lingered-in house anywhere.
            let (w, k) = uni
                .worlds
                .values()
                .flat_map(|w| w.heat.iter().map(move |(k, h)| (w.manifest.id.clone(), k.clone(), h.inside)))
                .max_by(|a, b| a.2.total_cmp(&b.2))
                .map(|(w, k, _)| (w, k))
                .unwrap_or(("the-lush".into(), "1,0".into()));
            let (x, z) = parse_addr(&k).unwrap_or((1, 0));
            (t.to_string(), i.to_string(), w, x, z)
        }
    };
    let slug: String = title.to_lowercase().chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '-' }).collect::<String>().split('-').filter(|s| !s.is_empty()).collect::<Vec<_>>().join("-");
    Some(Project { title, idea, world, x, z, slug: format!("{}-{:x}", slug.chars().take(40).collect::<String>(), now_ms() % 0xffff) })
}

fn brief(p: &Project, w: &WorldManifest, minutes: u32) -> String {
    format!(
        r##"You are working the night shift for Branches, a tranquil, slightly liminal world of identical houses whose insides grow. While everyone sleeps, build one complete, delightful game that will open as an attraction inside a house in "{world}" ({tagline}).

The project: {title}
{idea}

Build it in the current folder:
- index.html: a single self-contained file (inline CSS and JS; no network requests, no external libraries, no assets you cannot inline). Canvas 2D or WebGL are both fine.
- It opens full screen inside the world. Fit any size, look good from 960x600 up, and work with keyboard and mouse (and gamepad if you like). Start with a title screen that says how to play.
- Match this world's mood and palette: sky {sky}, horizon {horizon}, ground {ground}, accents {flowers}.
- Save progress in localStorage. Keep it under 2 MB.
- meta.json: {{"title": "...", "summary": "one sentence", "controls": "short"}}.

You have about {minutes} minutes. Go deep: depth of play, polish, small surprises. Test your logic as you go. Work only inside this folder."##,
        world = w.name,
        tagline = w.tagline,
        title = p.title,
        idea = p.idea,
        sky = w.generator.params["sky"]["top"],
        horizon = w.generator.params["sky"]["horizon"],
        ground = w.generator.params["ground"],
        flowers = w.generator.params["flowers"],
        minutes = minutes,
    )
}

fn run_agent(cmd: &str, dir: &Path, prompt: &str, max: Duration) -> Result<(), String> {
    use std::io::Write;
    use std::process::Stdio;
    let log = std::fs::File::create(dir.join("agent.log")).map_err(|e| e.to_string())?;
    let mut child = crate::agent::command(cmd).current_dir(dir).stdin(Stdio::piped()).stdout(log.try_clone().map_err(|e| e.to_string())?).stderr(log).spawn().map_err(|e| e.to_string())?;
    child.stdin.take().ok_or("no stdin")?.write_all(prompt.as_bytes()).map_err(|e| e.to_string())?;
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            return if status.success() { Ok(()) } else { Err(format!("the agent exited with {status}")) };
        }
        if started.elapsed() > max {
            let _ = child.kill();
            return Err("the agent ran out of time".into());
        }
        std::thread::sleep(Duration::from_secs(5));
    }
}

/// Build one project now (blocking). Returns its title.
pub fn work(app: &Arc<App>, p: Project) -> Result<String, String> {
    let night = app.settings.lock().unwrap().night.clone();
    let manifest = app.uni.lock().unwrap().worlds.get(&p.world).map(|w| w.manifest.clone()).ok_or("no such world")?;
    let rel = format!("worlds/{}/attractions/{}", p.world, p.slug);
    let dir = app.store.root.join(&rel);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let text = brief(&p, &manifest, night.max_minutes);
    std::fs::write(dir.join("BRIEF.md"), &text).map_err(|e| e.to_string())?;
    tracing::info!("night shift: {} in {}", p.title, dir.display());
    let result = run_agent(&night.command, &dir, &text, Duration::from_secs(night.max_minutes as u64 * 60));
    let index = dir.join("index.html");
    let size = std::fs::metadata(&index).map(|m| m.len()).unwrap_or(0);
    if size == 0 || size > 5_000_000 {
        return Err(result.err().unwrap_or_else(|| "the agent did not leave an index.html".into()));
    }
    let meta: serde_json::Value = std::fs::read(dir.join("meta.json")).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or(json!({}));
    let title = meta["title"].as_str().unwrap_or(&p.title).to_string();
    let now = now_ms();
    let mut uni = app.uni.lock().unwrap();
    let r = uni.room_mut(&p.world, p.x, p.z).ok_or("the house is gone")?;
    let chamber = r.growth();
    r.things.push(Thing {
        id: new_id(),
        kind: "attraction".into(),
        chamber,
        seed: (now & 0xffff_ffff) as u32,
        data: json!({ "title": title, "summary": meta["summary"], "controls": meta["controls"], "path": format!("{rel}/index.html"), "idea": p.idea }),
        by: "night shift".into(),
        at: now,
    });
    push_log(&mut r.log, LogEntry { id: new_id(), kind: "growth".into(), who: "the night shift".into(), player: None, text: format!("While everyone slept, the architect built {title}."), at: now });
    let investors: Vec<String> = uni.players.keys().cloned().collect();
    for pid in investors {
        if let Some(pl) = uni.players.get_mut(&pid) {
            pl.inbox.push(Notification { id: new_id(), at: now, world: p.world.clone(), x: p.x, z: p.z, text: format!("While you slept: {title} opened at {},{}.", p.x, p.z), read: false });
        }
    }
    drop(uni);
    app.note_commit(format!("Night shift: {title} ({rel})"));
    app.touch();
    app.broadcast(&p.world, &json!({ "t": "room", "addr": addr(p.x, p.z), "rebuild": true }));
    Ok(title)
}

fn record(app: &App, title: &str) {
    let path = app.store.root.join(".branches/night.json");
    let mut done: Vec<String> = std::fs::read(&path).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default();
    done.push(title.to_string());
    let _ = std::fs::write(&path, serde_json::to_vec_pretty(&done).unwrap_or_default());
}

pub fn built(app: &App) -> Vec<String> {
    std::fs::read(app.store.root.join(".branches/night.json")).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
}

/// Start a project in the background if one isn't running. `now` skips the
/// hours and idle checks (the "build something now" button).
pub fn start(app: &Arc<App>, now: bool) -> Option<String> {
    let night = app.settings.lock().unwrap().night.clone();
    if !now {
        let idle = app.last_activity.lock().unwrap().elapsed() > Duration::from_secs(night.idle_minutes as u64 * 60);
        let last = *app.last_night.lock().unwrap();
        let rested = last.is_none_or(|t| t.elapsed() > Duration::from_secs(10 * 3600));
        if !night.enabled || !idle || !rested || !in_window(&night, local_hour()) {
            return None;
        }
    }
    if app.night_running.swap(true, Ordering::SeqCst) {
        return None;
    }
    let project = {
        let uni = app.uni.lock().unwrap();
        choose(&uni, &built(app))
    };
    let Some(project) = project else {
        app.night_running.store(false, Ordering::SeqCst);
        return None;
    };
    let title = project.title.clone();
    *app.last_night.lock().unwrap() = Some(Instant::now());
    let app = app.clone();
    tokio::task::spawn_blocking(move || {
        let key = project.idea.clone();
        let name = project.title.clone();
        match work(&app, project) {
            Ok(t) => tracing::info!("night shift built {t}"),
            Err(e) => tracing::warn!("night shift on {name}: {e}"),
        }
        record(&app, if IDEAS.iter().any(|(t, _)| *t == name) { &name } else { &key });
        app.night_running.store(false, Ordering::SeqCst);
    });
    Some(title)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_wrap_midnight() {
        let n = Night { start_hour: 23, end_hour: 6, ..Default::default() };
        assert!(in_window(&n, 23) && in_window(&n, 2) && !in_window(&n, 12));
        let d = Night::default();
        assert!(in_window(&d, 3) && !in_window(&d, 7));
    }

    #[test]
    fn honours_game_requests_first() {
        let mut uni = Universe::default();
        uni.seed_builtins();
        let r = uni.room_mut("dusk-orchard", 2, 3).unwrap();
        r.log.push(LogEntry { id: "1".into(), kind: "request".into(), who: "ada".into(), player: None, text: "please build a go kart track game".into(), at: 5 });
        let p = choose(&uni, &[]).unwrap();
        assert_eq!((p.world.as_str(), p.x, p.z), ("dusk-orchard", 2, 3));
        let p2 = choose(&uni, &["please build a go kart track game".into()]).unwrap();
        assert_eq!(p2.title, "Zoo Tycoon");
    }
}

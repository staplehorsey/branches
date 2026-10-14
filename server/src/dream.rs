//! The dream: what steers the architect beyond one house.
//!
//! * Muses: links and words you love (a tweet, a story, a song). Each is
//!   read and distilled by Claude into an aesthetic, a few lines about its
//!   mood, light, materials and feeling.
//! * The compass: in any house you can say warmer or colder. Every mark
//!   keeps the house's idea, so the architect can see which ideas pull you
//!   in and which push you away, and step from the warm ones and away from
//!   the cold ones: a gradient descent on taste, one house at a time.
//! * Neighbours: a new house's idea is a variation on the ideas around it,
//!   a small step from a warm one, a bold leap from a cold one, so walking
//!   down a street wanders through nearby aesthetics.
//!
//! All of it lives in `.branches/dream.json` with your worlds.

use crate::model::*;
use crate::state::App;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Muse {
    pub id: String,
    #[serde(default)]
    pub url: Option<String>,
    /// What was read from the link, or what you wrote.
    pub source: String,
    pub name: String,
    pub essence: String,
    pub at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mark {
    pub world: String,
    pub x: i32,
    pub z: i32,
    pub idea: String,
    pub premise: String,
    /// +1 warmer, -1 colder, 0 just a note.
    pub score: i32,
    pub at: u64,
    /// Which space it was said in (none: the house itself), and any words.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub space: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Dream {
    #[serde(default)]
    pub muses: Vec<Muse>,
    #[serde(default)]
    pub compass: Vec<Mark>,
}

static LOCK: Mutex<()> = Mutex::new(());

fn file(root: &Path) -> PathBuf {
    root.join(".branches/dream.json")
}

pub fn load(root: &Path) -> Dream {
    std::fs::read(file(root)).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
}

fn update<T>(root: &Path, f: impl FnOnce(&mut Dream) -> T) -> T {
    let _g = LOCK.lock().unwrap();
    let mut d = load(root);
    let out = f(&mut d);
    let _ = std::fs::create_dir_all(root.join(".branches"));
    let _ = std::fs::write(file(root), serde_json::to_vec_pretty(&d).unwrap_or_default());
    out
}

// ------------------------------------------------------------------ muses

fn strip_tags(html: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            c if !in_tag => out.push(c),
            _ => {}
        }
    }
    let out = out.replace("&amp;", "&").replace("&quot;", "\"").replace("&#39;", "'").replace("&lt;", "<").replace("&gt;", ">").replace("&nbsp;", " ").replace("&mdash;", "—");
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Drop whole elements whose text is never the content (scripts, styles).
fn drop_blocks(html: &str, tag: &str) -> String {
    let lower = html.to_lowercase();
    let (open, close) = (format!("<{tag}"), format!("</{tag}>"));
    let mut out = String::new();
    let mut i = 0;
    while let Some(a) = lower[i..].find(&open) {
        out.push_str(&html[i..i + a]);
        match lower[i + a..].find(&close) {
            Some(b) => i = i + a + b + close.len(),
            None => return out,
        }
    }
    out.push_str(&html[i..]);
    out
}

fn meta(html: &str, key: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let at = lower.find(&format!("\"{key}\""))?;
    let start = lower[..at].rfind('<')?;
    let end = start + lower[start..].find('>')?;
    let tag = &html[start..end];
    let c = tag.to_lowercase().find("content=")? + 8;
    let q = tag[c..].chars().next()?;
    let rest = &tag[c + 1..];
    Some(strip_tags(&rest[..rest.find(q)?]))
}

/// Read a link: a tweet through X's public embed, anything else as a page.
fn read_link(url: &str) -> Result<String, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder().timeout_global(Some(std::time::Duration::from_secs(20))).build().into();
    let get = |u: &str| -> Result<String, String> {
        let mut r = agent.get(u).header("user-agent", "Mozilla/5.0 (Macintosh) Branches").call().map_err(|e| e.to_string())?;
        r.body_mut().with_config().limit(2_000_000).read_to_string().map_err(|e| e.to_string())
    };
    let host = url.split("//").nth(1).unwrap_or("").split('/').next().unwrap_or("").to_lowercase();
    if ["twitter.com", "www.twitter.com", "x.com", "www.x.com", "mobile.twitter.com"].contains(&host.as_str()) {
        let body = get(&format!("https://publish.x.com/oembed?omit_script=true&url={}", urlencode(url)))?;
        let v: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
        return Ok(format!("A post by {}: {}", v["author_name"].as_str().unwrap_or("someone"), strip_tags(v["html"].as_str().unwrap_or(""))));
    }
    let html = get(url)?;
    let title = meta(&html, "og:title").or_else(|| {
        let l = html.to_lowercase();
        let a = l.find("<title")?;
        let a = a + l[a..].find('>')? + 1;
        Some(strip_tags(&html[a..a + l[a..].find("</title")?]))
    });
    let desc = meta(&html, "og:description").or_else(|| meta(&html, "description"));
    let mut body = html.clone();
    for t in ["script", "style", "noscript", "svg", "nav", "header", "footer"] {
        body = drop_blocks(&body, t);
    }
    let text: String = strip_tags(&body).chars().take(3000).collect();
    Ok(format!("Title: {}\nDescription: {}\nText: {}", title.unwrap_or_default(), desc.unwrap_or_default(), text))
}

fn urlencode(s: &str) -> String {
    s.bytes().map(|b| if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) { (b as char).to_string() } else { format!("%{b:02X}") }).collect()
}

/// The program that distills muses: your architect, or Claude Code.
fn thinker(app: &App) -> Option<String> {
    let s = app.settings.lock().unwrap().clone();
    if s.architect == "command" && !s.command.trim().is_empty() {
        Some(s.command)
    } else {
        crate::agent::find_claude().map(|_| "claude -p".to_string())
    }
}

/// Add a muse: read the link (if any), then distill its aesthetic.
pub fn add_muse(app: &App, url: Option<String>, words: String) -> Result<Muse, String> {
    let url = url.map(|u| u.trim().to_string()).filter(|u| !u.is_empty());
    if let Some(u) = &url {
        if !(u.starts_with("http://") || u.starts_with("https://")) {
            return Err("links start with http:// or https://".into());
        }
    }
    let mut source = words.trim().to_string();
    if let Some(u) = &url {
        match read_link(u) {
            Ok(t) => source = format!("{source}\n{t}").trim().to_string(),
            Err(e) if source.is_empty() => return Err(format!("couldn't read that link ({e}); paste the words instead")),
            Err(_) => {}
        }
    }
    if source.is_empty() {
        return Err("give a link or some words".into());
    }
    let source: String = source.chars().take(4000).collect();
    let (name, essence) = match thinker(app) {
        Some(cmd) => {
            let prompt = format!(
                "Someone building a dream world to wander through loves this{link}:\n\n{source}\n\nDistill its aesthetic so an architect can build rooms from it. Reply with ONLY JSON: {{\"name\": \"3-6 evocative words\", \"essence\": \"80-120 words: its mood and emotional register, its light, colours, materials, textures, sounds, recurring motifs, scale, and what it asks you to pay attention to\"}}",
                link = url.as_ref().map(|u| format!(" ({u})")).unwrap_or_default()
            );
            let out = crate::agent::run(&cmd, &prompt, Some(&crate::agent::workdir(&app.store.root)), std::time::Duration::from_secs(180))?;
            let v = crate::architect::extract_json(&out).ok_or("Claude didn't answer in JSON")?;
            (v["name"].as_str().unwrap_or("A muse").to_string(), v["essence"].as_str().unwrap_or("").to_string())
        }
        None => (source.chars().take(40).collect(), source.clone()),
    };
    let m = Muse { id: new_id(), url, source, name: name.chars().take(60).collect(), essence: essence.chars().take(1200).collect(), at: now_ms() };
    update(&app.store.root, |d| d.muses.push(m.clone()));
    Ok(m)
}

pub fn remove_muse(app: &App, id: &str) {
    update(&app.store.root, |d| d.muses.retain(|m| m.id != id));
}

// ---------------------------------------------------------------- compass

/// Warmer (+1) or colder (-1), here. Warmer also gives the house a push
/// toward its next refinement.
pub fn mark(app: &App, world: &str, x: i32, z: i32, score: i32, space: Option<u32>, note: Option<String>) -> Result<Value, String> {
    let score = score.signum();
    let note = note.map(|n| n.trim().chars().take(500).collect::<String>()).filter(|n| !n.is_empty());
    if score == 0 && note.is_none() {
        return Err("warmer, colder, or a few words".into());
    }
    let now = now_ms();
    let (idea, premise, name) = {
        let mut uni = app.uni.lock().unwrap();
        let threshold = app.threshold(uni.room(world, x, z).map(|r| r.growth()).unwrap_or(0));
        let r = uni.room_mut(world, x, z).ok_or("no house there")?;
        let (mut idea, mut premise) = match &r.idea {
            Some(i) => (i.name.clone(), i.premise.clone()),
            None => (procgen::theme(&r.theme).map(|t| t.name).unwrap_or("this house").to_string(), "(no idea yet: only its starting theme)".to_string()),
        };
        if let Some(s) = space.and_then(|n| r.spaces.iter().find(|s| s.n == n)) {
            idea = format!("{} (space {}: {})", idea, s.n, s.name);
            premise = format!("{} This space: {}", premise, s.summary);
        }
        if score > 0 {
            r.attention += threshold * 0.35;
        }
        let said = match (score, &note) {
            (1, Some(n)) => format!("warmer: {n}"),
            (-1, Some(n)) => format!("colder: {n}"),
            (_, Some(n)) => n.clone(),
            (1, None) => "warmer".into(),
            _ => "colder".into(),
        };
        push_log(&mut r.log, LogEntry { id: new_id(), kind: "compass".into(), who: "the dreamer".into(), player: None, text: said, at: now });
        (idea.clone(), premise, idea)
    };
    app.touch();
    update(&app.store.root, |d| {
        d.compass.push(Mark { world: world.into(), x, z, idea, premise, score, at: now, space, note: note.clone() });
        let n = d.compass.len();
        if n > 400 {
            d.compass.drain(..n - 400);
        }
    });
    Ok(json!({ "ok": true, "idea": name, "score": score }))
}

use crate::procgen;

// ---------------------------------------------------------- for the prompt

/// Everything beyond this house that should steer its idea, in words.
pub fn guidance(root: &Path, uni: &Universe, world: &str, x: i32, z: i32) -> String {
    let d = load(root);
    let mut out = String::new();
    if !d.muses.is_empty() {
        out.push_str("The dreamer's muses (things they love; build from their aesthetics, never literally illustrate them):\n");
        for m in d.muses.iter().rev().take(8) {
            out.push_str(&format!("- {}: {}\n", m.name, m.essence));
        }
    }
    // Net feeling per idea, most recent first.
    let mut felt: Vec<(String, String, i32, u64)> = Vec::new();
    for m in d.compass.iter().rev() {
        match felt.iter_mut().find(|f| f.0 == m.idea) {
            Some(f) => f.2 += m.score,
            None => felt.push((m.idea.clone(), m.premise.clone(), m.score, m.at)),
        }
    }
    let notes: Vec<String> = d.compass.iter().rev().filter_map(|m| m.note.as_ref().map(|n| format!("- on {}: \"{n}\"", m.idea))).take(8).collect();
    if !notes.is_empty() {
        out.push_str("What the dreamer said, in their words (most recent first):\n");
        out.push_str(&notes.join("\n"));
        out.push('\n');
    }
    let warm: Vec<_> = felt.iter().filter(|f| f.2 > 0).take(8).collect();
    let cold: Vec<_> = felt.iter().filter(|f| f.2 < 0).take(6).collect();
    if !warm.is_empty() {
        out.push_str("Warmer (ideas that pulled the dreamer in, most recent first):\n");
        for f in &warm {
            out.push_str(&format!("- {} (+{}): {}\n", f.0, f.2, f.1));
        }
    }
    if !cold.is_empty() {
        out.push_str("Colder (ideas that pushed the dreamer away):\n");
        for f in &cold {
            out.push_str(&format!("- {} ({}): {}\n", f.0, f.2, f.1));
        }
    }
    let recent: Vec<i32> = d.compass.iter().rev().take(6).map(|m| m.score).collect();
    if !recent.is_empty() {
        let t: i32 = recent.iter().sum();
        out.push_str(&match t {
            t if t >= 3 => "They have been saying warmer: they are close. Take small, precise steps from the warm ideas; refine, don't wander.\n".to_string(),
            t if t <= -2 => "They have been saying colder: take a bold leap away from the cold ideas, somewhere new.\n".to_string(),
            _ => "Their feedback is mixed: step from the warm ideas, in a direction not yet tried.\n".to_string(),
        });
    }
    if let Some(w) = uni.worlds.get(world) {
        let mut near: Vec<(i32, String)> = Vec::new();
        for r in w.rooms.values() {
            let dist = (r.x - x).abs().max((r.z - z).abs());
            if dist == 0 || dist > 3 {
                continue;
            }
            if let Some(i) = &r.idea {
                let feel: i32 = d.compass.iter().filter(|m| m.world == world && m.x == r.x && m.z == r.z).map(|m| m.score).sum();
                let feel = match feel {
                    f if f > 0 => " (warm)",
                    f if f < 0 => " (cold)",
                    _ => "",
                };
                near.push((dist, format!("- {} houses away at {},{}: {}{feel}: {}\n", dist, r.x, r.z, i.name, i.premise)));
            }
        }
        near.sort_by_key(|n| n.0);
        if !near.is_empty() {
            out.push_str("Neighbouring houses' ideas (this street should feel like wandering through a dream: a neighbour is a close variation, further away the ideas drift):\n");
            for (_, line) in near.into_iter().take(6) {
                out.push_str(&line);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_pages() {
        let html = r#"<html><head><title>Jolene</title><meta property="og:description" content="I&#39;m begging of you"><script>var x = "<p>no</p>";</script></head><body><p>Please don't take him</p></body></html>"#;
        assert_eq!(meta(html, "og:description").unwrap(), "I'm begging of you");
        let body = drop_blocks(html, "script");
        assert!(!body.contains("var x"));
        assert!(strip_tags(&body).contains("Please don't take him"));
        assert_eq!(urlencode("https://x.com/a?b=c"), "https%3A%2F%2Fx.com%2Fa%3Fb%3Dc");
    }
}

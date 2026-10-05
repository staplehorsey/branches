//! The architect: an asynchronous builder that grows rooms where people
//! spend their time, biased toward what those people seem to want.
//!
//! Signals it learns from:
//!   * dwell: seconds in a chamber reinforce that chamber's tags,
//!   * gaze: seconds spent looking at a feature or thing reinforce its tags,
//!   * use: talking to characters, reading notes, playing games,
//!   * admiration: "more of this",
//!   * requests: visitor-book text, mined for tags (and passed verbatim to
//!     creative architects).
//!
//! How much it builds adapts to how people explore. A world whose visitors
//! hurry from house to house gets quick sketches (cheap, sooner, more of
//! them); one where they linger gets rich builds (a creative architect,
//! more features and things, detail added to what is already there).
//!
//! `Architect` is the seam: the built-in `Heuristic`, or `Command`, which
//! hands a prompt to any program (`claude -p`, a local model, a script) and
//! reads back a JSON plan. Builds run off the main loop, so a slow creative
//! architect never stalls the world.

use crate::model::*;
use crate::procgen::{self, Rng};
use crate::state::{App, room_summary};
use crate::stories::{self, WorldStory};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Budget {
    Sketch,
    Normal,
    Rich,
}

impl Budget {
    pub fn name(self) -> &'static str {
        match self {
            Budget::Sketch => "sketch",
            Budget::Normal => "normal",
            Budget::Rich => "rich",
        }
    }
    /// From the world's average seconds per house visit.
    pub fn from_pace(dwell: f32) -> Self {
        if dwell <= 0.0 {
            Budget::Normal
        } else if dwell < 25.0 {
            Budget::Sketch
        } else if dwell > 90.0 {
            Budget::Rich
        } else {
            Budget::Normal
        }
    }
    /// Multiplier on the attention a growth costs.
    pub fn cost(self) -> f64 {
        match self {
            Budget::Sketch => 0.6,
            Budget::Normal => 1.0,
            Budget::Rich => 1.4,
        }
    }
}

#[derive(Clone)]
pub struct PlanContext {
    pub world: String,
    pub world_name: String,
    pub room: Room,
    /// Blended desire per tag, normalised to sum 1.
    pub desire: BTreeMap<Tag, f32>,
    pub requests: Vec<String>,
    pub notes: Vec<String>,
    /// Most looked-at and most used features/things, as readable labels.
    pub looked: Vec<(String, f32)>,
    pub used: Vec<(String, u32)>,
    pub story: WorldStoryOwned,
    pub budget: Budget,
    /// Add detail to existing chambers instead of growing a new one.
    pub detail: bool,
}

#[derive(Clone, Default)]
pub struct WorldStoryOwned {
    pub cast_here: Vec<String>,
    pub pages_placed: Vec<(String, usize)>,
    pub cat_here: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Plan {
    pub chamber_name: String,
    pub chamber_tag: Tag,
    pub features: Vec<Tag>,
    pub enrich: Vec<(u32, Tag)>,
    /// (kind, data, chamber); chamber None means the new chamber.
    pub things: Vec<(String, Value, Option<u32>)>,
    pub note: Option<String>,
    pub by: String,
}

pub trait Architect: Send + Sync {
    fn id(&self) -> String;
    fn plan(&self, ctx: &PlanContext, seed: u64) -> Plan;
}

pub struct Heuristic;

fn sample(desire: &BTreeMap<Tag, f32>, rng: &mut Rng, novelty: f32) -> Tag {
    if rng.f32() < novelty || desire.is_empty() {
        return rng.pick(procgen::TAGS).to_string();
    }
    // Sharpen toward strong preferences.
    let total: f32 = desire.values().map(|w| w.powf(1.5)).sum();
    let mut roll = rng.f32() * total;
    for (tag, w) in desire {
        roll -= w.powf(1.5);
        if roll <= 0.0 {
            return tag.clone();
        }
    }
    desire.keys().next_back().cloned().unwrap_or_else(|| "light".into())
}

impl Architect for Heuristic {
    fn id(&self) -> String {
        "reference-heuristic@2".into()
    }

    fn plan(&self, ctx: &PlanContext, seed: u64) -> Plan {
        let mut rng = Rng::new(seed);
        let requested: Vec<&str> = ctx.requests.iter().take(2).flat_map(|t| procgen::tags_in_text(t)).collect();
        let chamber_tag = if !requested.is_empty() && rng.f32() < 0.5 {
            rng.pick(&requested).to_string()
        } else {
            sample(&ctx.desire, &mut rng, 0.12)
        };
        let (n_features, n_things, enrich_p) = match ctx.budget {
            Budget::Sketch => (1, if rng.f32() < 0.35 { 1 } else { 0 }, 0.0),
            Budget::Normal => (2 + (rng.next_u64() % 2) as usize, 1, 0.5),
            Budget::Rich => (3 + (rng.next_u64() % 2) as usize, 2, 1.0),
        };
        let chambers = ctx.room.chambers.len() as u32;
        let story = WorldStory { cast_here: ctx.story.cast_here.clone(), pages_placed: ctx.story.pages_placed.clone(), cat_here: ctx.story.cat_here };
        let mut plan = Plan { by: self.id(), ..Default::default() };
        if ctx.detail {
            // Deepen what is already there: the busiest chambers first.
            for _ in 0..n_features + 1 {
                plan.enrich.push(((rng.next_u64() % chambers as u64) as u32, sample(&ctx.desire, &mut rng, 0.1)));
            }
            for _ in 0..n_things.max(1) {
                let tag = sample(&ctx.desire, &mut rng, 0.15);
                if let Some((k, d)) = stories::pick(&tag, &story, &mut rng) {
                    plan.things.push((k, d, Some((rng.next_u64() % chambers as u64) as u32)));
                }
            }
            plan.chamber_tag = chamber_tag;
            return plan;
        }
        plan.features.push(chamber_tag.clone());
        for _ in 1..n_features {
            plan.features.push(sample(&ctx.desire, &mut rng, 0.2));
        }
        if chambers > 1 && rng.f32() < enrich_p {
            plan.enrich.push(((rng.next_u64() % chambers as u64) as u32, sample(&ctx.desire, &mut rng, 0.1)));
        }
        for i in 0..n_things {
            let tag = if i == 0 { chamber_tag.clone() } else { sample(&ctx.desire, &mut rng, 0.2) };
            if let Some((k, d)) = stories::pick(&tag, &story, &mut rng) {
                plan.things.push((k, d, None));
            }
        }
        plan.chamber_name = procgen::chamber_name(&chamber_tag, &mut rng);
        plan.chamber_tag = chamber_tag;
        plan
    }
}

/// Any program as architect: it gets a prompt on stdin and prints a JSON
/// plan. `claude -p` makes Claude Code the architect.
pub struct Command {
    pub cmd: String,
}

#[derive(Deserialize)]
struct RawPlan {
    #[serde(default)]
    chamber_name: Option<String>,
    #[serde(default)]
    chamber_tag: Option<String>,
    #[serde(default)]
    features: Vec<String>,
    #[serde(default)]
    enrich: Vec<RawEnrich>,
    #[serde(default)]
    things: Vec<RawThing>,
    #[serde(default)]
    note: Option<String>,
}
#[derive(Deserialize)]
struct RawEnrich {
    chamber: u32,
    tag: String,
}
#[derive(Deserialize)]
struct RawThing {
    kind: String,
    #[serde(default)]
    chamber: Option<u32>,
    #[serde(default)]
    data: Value,
}

pub fn prompt(ctx: &PlanContext) -> String {
    let r = &ctx.room;
    let chambers: Vec<String> = r.chambers.iter().enumerate().map(|(i, c)| format!("{i}: {} ({})", c.name, c.tag)).collect();
    let mut feats: BTreeMap<&str, u32> = BTreeMap::new();
    for f in &r.features {
        *feats.entry(&f.tag).or_default() += 1;
    }
    let things: Vec<String> = r.things.iter().map(|t| format!("{} in chamber {}: {}", t.kind, t.chamber, t.data.get("name").or(t.data.get("title")).and_then(|v| v.as_str()).unwrap_or(""))).collect();
    format!(
        r##"You are the architect of Branches, an endless, tranquil, slightly liminal world: streets of identical houses whose insides grow as people linger. Design the next addition to one house. Be creative, warm and specific. Reply with ONLY one JSON object, no prose.

World: {world}. House at {x},{z}, theme {theme}.
Chambers so far: {chambers}
Features so far (tag counts): {feats:?}
Things living here: {things:?}
What visitors want (tag weights): {desire:?}
What they look at most: {looked:?}
What they use most: {used:?}
Fresh requests in the visitor book: {requests:?}
Recent visitor-book notes: {notes:?}
Budget: {budget}. Mode: {mode}.

Feature tags the renderer knows: {tags}.
Thing kinds the renderer knows (invent freely within these shapes):
  character {{"name": str, "lines": [3-6 short spoken lines, in order], "look": {{"coat": "#hex", "accent": "#hex", "hat": one of wide|cap|visor|straw|tall|none|scarf|beret|beanie|bowler|net}}}}
  note {{"title": str, "text": str}}
  lanterns {{"count": 3-6, "reward": str}}
  bells {{"order": three of low|middle|high, "hint": str}}
  cat {{"name": str, "says": str}}
Characters may mention other characters and other worlds (The Lush, Dusk Orchard, Fog Pines, Salt Flat Noon, Moonlit Meadow) so stories stitch together. Continue stories already here rather than repeating them.
Story kit you may continue: {kit}

JSON shape:
{{"chamber_name": "The ...", "chamber_tag": "<tag>", "features": ["<tag>", ...], "enrich": [{{"chamber": 0, "tag": "<tag>"}}], "things": [{{"kind": "...", "chamber": null, "data": {{...}}}}], "note": "optional one line for the visitor book, from the architect"}}
In "detail" mode leave chamber_name empty and put everything in enrich/things with explicit chamber numbers."##,
        world = ctx.world_name,
        x = r.x,
        z = r.z,
        theme = r.theme,
        chambers = chambers.join("; "),
        feats = feats,
        things = things,
        desire = ctx.desire.iter().map(|(k, v)| format!("{k}:{v:.2}")).collect::<Vec<_>>(),
        looked = ctx.looked,
        used = ctx.used,
        requests = ctx.requests,
        notes = ctx.notes,
        budget = ctx.budget.name(),
        mode = if ctx.detail { "detail: deepen existing chambers" } else { "grow one new chamber" },
        tags = procgen::TAGS.join(", "),
        kit = stories::prompt_summary(),
    )
}

/// Pull the first balanced JSON object out of a model's reply.
fn extract_json(text: &str) -> Option<Value> {
    let start = text.find('{')?;
    let mut depth = 0i32;
    let mut in_str = false;
    let mut esc = false;
    for (i, ch) in text[start..].char_indices() {
        if in_str {
            match ch {
                '\\' if !esc => esc = true,
                '"' if !esc => in_str = false,
                _ => esc = false,
            }
            continue;
        }
        match ch {
            '"' => in_str = true,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return serde_json::from_str(&text[start..start + i + 1]).ok();
                }
            }
            _ => {}
        }
    }
    None
}

pub fn parse_plan(text: &str, by: &str) -> Option<Plan> {
    let raw: RawPlan = serde_json::from_value(extract_json(text)?).ok()?;
    let known = |t: &String| procgen::TAGS.contains(&t.as_str());
    let tag = raw.chamber_tag.filter(known).unwrap_or_else(|| "light".into());
    Some(Plan {
        chamber_name: raw.chamber_name.unwrap_or_default().chars().take(60).collect(),
        chamber_tag: tag,
        features: raw.features.into_iter().filter(known).take(6).collect(),
        enrich: raw.enrich.into_iter().filter(|e| known(&e.tag)).take(6).map(|e| (e.chamber, e.tag)).collect(),
        things: raw.things.into_iter().take(4).filter(|t| !t.kind.is_empty() && t.kind.len() < 24).map(|t| (t.kind, t.data, t.chamber)).collect(),
        note: raw.note.map(|n| n.chars().take(280).collect()),
        by: by.into(),
    })
}

impl Architect for Command {
    fn id(&self) -> String {
        format!("command:{}", self.cmd.split_whitespace().next().unwrap_or("?"))
    }

    fn plan(&self, ctx: &PlanContext, seed: u64) -> Plan {
        use std::io::Write;
        use std::process::{Command as Proc, Stdio};
        let run = || -> Option<Plan> {
            let mut child = Proc::new("/bin/sh").arg("-c").arg(&self.cmd).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().ok()?;
            child.stdin.take()?.write_all(prompt(ctx).as_bytes()).ok()?;
            let out = child.wait_with_output().ok()?;
            parse_plan(&String::from_utf8_lossy(&out.stdout), &self.id())
        };
        match run() {
            Some(p) if ctx.detail || !p.chamber_name.is_empty() => p,
            Some(mut p) => {
                p.chamber_name = procgen::chamber_name(&p.chamber_tag, &mut Rng::new(seed));
                p
            }
            None => {
                tracing::warn!("architect command `{}` gave no usable plan; using the heuristic", self.cmd);
                Heuristic.plan(ctx, seed)
            }
        }
    }
}

/// Tags a chamber is "about": its own tag plus what has been built in it.
pub fn chamber_tags(room: &Room, chamber: u32) -> Vec<Tag> {
    let mut tags: Vec<Tag> = room.chambers.get(chamber as usize).map(|c| vec![c.tag.clone()]).unwrap_or_default();
    if chamber == 0 {
        if let Some(t) = procgen::theme(&room.theme) {
            tags.extend(t.tags.iter().map(|s| s.to_string()));
        }
    }
    tags.extend(room.features.iter().filter(|f| f.chamber == chamber).map(|f| f.tag.clone()));
    tags.sort();
    tags.dedup();
    tags
}

/// Blend room weights with investors' taste, normalised.
pub fn desire(room: &Room, uni: &Universe) -> BTreeMap<Tag, f32> {
    let mut out: BTreeMap<Tag, f32> = BTreeMap::new();
    let room_total: f32 = room.weights.values().sum::<f32>().max(1e-3);
    for (t, w) in &room.weights {
        *out.entry(t.clone()).or_default() += 0.55 * w / room_total;
    }
    let invested: f64 = room.investors.values().sum::<f64>().max(1e-3);
    for (pid, secs) in &room.investors {
        let Some(p) = uni.players.get(pid) else { continue };
        let ptotal: f32 = p.prefs.values().sum::<f32>().max(1e-3);
        let share = (secs / invested) as f32;
        for (t, w) in &p.prefs {
            *out.entry(t.clone()).or_default() += 0.45 * share * w / ptotal;
        }
    }
    let total: f32 = out.values().sum::<f32>().max(1e-3);
    out.values_mut().for_each(|v| *v /= total);
    out
}

pub fn leaning(room: &Room, uni: &Universe, n: usize) -> Vec<Tag> {
    let mut d: Vec<(Tag, f32)> = desire(room, uni).into_iter().collect();
    d.sort_by(|a, b| b.1.total_cmp(&a.1));
    d.into_iter().take(n).map(|(t, _)| t).collect()
}

fn bump(map: &mut BTreeMap<Tag, f32>, tag: &str, by: f32) {
    let v = map.entry(tag.to_string()).or_default();
    *v = (*v + by).min(50.0);
}

/// Record `dt` seconds of a player standing in a chamber.
pub fn record_dwell(uni: &mut Universe, world: &str, x: i32, z: i32, chamber: u32, pid: &str, dt: f64) {
    let Some(room) = uni.room_mut(world, x, z) else { return };
    room.attention += dt;
    *room.investors.entry(pid.to_string()).or_default() += dt;
    let tags = chamber_tags(room, chamber);
    for t in &tags {
        bump(&mut room.weights, t, 0.01 * dt as f32);
    }
    if let Some(p) = uni.players.get_mut(pid) {
        for t in &tags {
            bump(&mut p.prefs, t, 0.01 * dt as f32);
        }
    }
}

/// "More of this": a strong, explicit signal.
pub fn record_admire(uni: &mut Universe, world: &str, x: i32, z: i32, chamber: u32, pid: &str) -> Vec<Tag> {
    let Some(room) = uni.room_mut(world, x, z) else { return vec![] };
    room.attention += 4.0;
    *room.investors.entry(pid.to_string()).or_default() += 4.0;
    let tags = chamber_tags(room, chamber);
    for t in &tags {
        bump(&mut room.weights, t, 0.6);
    }
    if let Some(p) = uni.players.get_mut(pid) {
        for t in &tags {
            bump(&mut p.prefs, t, 0.6);
        }
    }
    tags
}

/// Requests in the visitor log steer the room directly.
pub fn record_request(room: &mut Room, text: &str) -> Vec<&'static str> {
    let tags = procgen::tags_in_text(text);
    for t in &tags {
        bump(&mut room.weights, t, 3.0);
    }
    room.attention += 6.0;
    tags
}

/// One architect step, every second: start builds where attention crossed
/// the line, and hand finished timers to an architect off the main loop.
pub fn tick(app: &Arc<App>) {
    let now = now_ms();
    let mut broadcasts = Vec::new();
    let mut jobs = Vec::new();
    {
        let mut uni = app.uni.lock().unwrap();
        let world_ids: Vec<String> = uni.worlds.keys().cloned().collect();
        for wid in world_ids {
            let budget = Budget::from_pace(uni.worlds[&wid].pace);
            let keys: Vec<String> = uni.worlds[&wid].rooms.keys().cloned().collect();
            for key in keys {
                let (growth, attention, building) = {
                    let r = &uni.worlds[&wid].rooms[&key];
                    (r.growth(), r.attention, r.building.clone())
                };
                match building {
                    None if attention >= app.threshold(growth) * budget.cost() => {
                        let r = uni.worlds.get_mut(&wid).unwrap().rooms.get_mut(&key).unwrap();
                        let (lo, span) = match budget {
                            Budget::Sketch => (5, 8),
                            Budget::Normal => (12, 18),
                            Budget::Rich => (20, 25),
                        };
                        let secs = lo + (procgen::splitmix(r.seed as u64 ^ now) % span);
                        r.building = Some(Build { started: now, ready_at: now + secs * 1000 });
                        broadcasts.push((wid.clone(), json!({ "t": "room", "addr": key, "summary": room_summary(r) })));
                        app.touch();
                    }
                    Some(b) if now >= b.ready_at => {
                        let job = format!("{wid}/{key}");
                        if app.in_flight.lock().unwrap().insert(job.clone()) {
                            jobs.push((job, context(app, &uni, &wid, &key, budget)));
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    for (w, m) in broadcasts {
        app.broadcast(&w, &m);
    }
    for (job, ctx) in jobs {
        let app = app.clone();
        tokio::spawn(async move {
            let seed = procgen::splitmix(ctx.room.seed as u64 ^ ((ctx.room.chambers.len() as u64) << 32) ^ now_ms());
            let architect = app.architect_for(ctx.budget);
            let c = ctx.clone();
            let plan = tokio::task::spawn_blocking(move || architect.plan(&c, seed)).await.unwrap_or_else(|_| Heuristic.plan(&ctx, seed));
            apply(&app, &ctx, plan, seed);
            app.in_flight.lock().unwrap().remove(&job);
        });
    }
}

fn label(r: &Room, id: &str) -> String {
    if let Some(f) = r.features.iter().find(|f| f.id == id) {
        return format!("{} (feature, chamber {})", f.tag, f.chamber);
    }
    if let Some(t) = r.things.iter().find(|t| t.id == id) {
        let name = t.data.get("name").or(t.data.get("title")).and_then(|v| v.as_str()).unwrap_or("");
        return format!("{} {name} (chamber {})", t.kind, t.chamber);
    }
    if id == "guestbook" { "the visitor book".into() } else { id.into() }
}

fn context(_app: &App, uni: &Universe, wid: &str, key: &str, budget: Budget) -> PlanContext {
    let world = &uni.worlds[wid];
    let room = world.rooms[key].clone();
    let last_built = room.chambers.last().map(|c| c.at).unwrap_or(0);
    let mut looked: Vec<(String, f32)> = room.looks.iter().map(|(id, s)| (label(&room, id), (*s * 10.0).round() / 10.0)).collect();
    looked.sort_by(|a, b| b.1.total_cmp(&a.1));
    looked.truncate(6);
    let mut used: Vec<(String, u32)> = room.touches.iter().map(|(id, n)| (label(&room, id), *n)).collect();
    used.sort_by(|a, b| b.1.cmp(&a.1));
    used.truncate(6);
    let mut story = WorldStoryOwned::default();
    for r in world.rooms.values() {
        for t in &r.things {
            if let Some(c) = t.data.get("cast").and_then(|v| v.as_str()) {
                story.cast_here.push(c.into());
            }
            if let (Some(th), Some(p)) = (t.data.get("thread").and_then(|v| v.as_str()), t.data.get("page").and_then(|v| v.as_u64())) {
                story.pages_placed.push((th.into(), p as usize));
            }
            story.cat_here |= t.kind == "cat";
        }
    }
    // Near spawn, well-loved houses get deepened as often as they grow.
    let spawn = parse_addr(&world.manifest.spawn).unwrap_or((0, 0));
    let near = (room.x - spawn.0).abs().max((room.z - spawn.1).abs()) <= 3;
    // Alternate: after a new chamber, a pass of detail.
    let last_chamber = room.chambers.last().map(|c| c.at).unwrap_or(0);
    let last_detail = room.features.iter().chain(std::iter::empty()).map(|f| f.at).filter(|t| *t > last_chamber).max();
    let detail = near && room.growth() >= 2 && last_detail.is_none();
    PlanContext {
        world: wid.into(),
        world_name: world.manifest.name.clone(),
        desire: desire_with_signals(&room, uni),
        requests: room.log.iter().rev().filter(|l| l.kind == "request" && l.at > last_built).take(5).map(|l| l.text.clone()).collect(),
        notes: room.log.iter().rev().filter(|l| l.kind == "note" || l.kind == "praise").take(5).map(|l| l.text.clone()).collect(),
        looked,
        used,
        story,
        budget,
        detail,
        room,
    }
}

/// Desire, plus the tags of what people look at and use.
fn desire_with_signals(room: &Room, uni: &Universe) -> BTreeMap<Tag, f32> {
    let mut d = desire(room, uni);
    let mut extra: BTreeMap<Tag, f32> = BTreeMap::new();
    for (id, secs) in &room.looks {
        if let Some(f) = room.features.iter().find(|f| &f.id == id) {
            *extra.entry(f.tag.clone()).or_default() += secs.min(120.0) / 120.0;
        }
    }
    for (id, n) in &room.touches {
        if let Some(f) = room.features.iter().find(|f| &f.id == id) {
            *extra.entry(f.tag.clone()).or_default() += *n as f32 * 0.2;
        }
    }
    let total: f32 = extra.values().sum();
    if total > 0.0 {
        for (t, v) in extra {
            *d.entry(t).or_default() += 0.3 * v / total;
        }
        let sum: f32 = d.values().sum::<f32>().max(1e-3);
        d.values_mut().for_each(|v| *v /= sum);
    }
    d
}

fn apply(app: &Arc<App>, ctx: &PlanContext, plan: Plan, seed: u64) {
    let now = now_ms();
    let (wid, key) = (ctx.world.as_str(), crate::model::addr(ctx.room.x, ctx.room.z));
    let mut uni = app.uni.lock().unwrap();
    let world_name = uni.worlds[wid].manifest.name.clone();
    // Building on a house seen at an older version brings that version forward.
    let replunged_from = app.replunge(&mut uni, wid, ctx.room.x, ctx.room.z);
    let threshold = app.threshold(ctx.room.growth()) * ctx.budget.cost();
    let Some(r) = uni.worlds.get_mut(wid).and_then(|w| w.rooms.get_mut(&key)) else { return };
    let mut rng = Rng::new(seed);
    let by = plan.by.clone();
    let chambers_before = r.chambers.len() as u32;
    let new_chamber = !ctx.detail && !plan.chamber_name.is_empty();
    if new_chamber {
        r.chambers.push(Chamber { name: plan.chamber_name.clone(), tag: plan.chamber_tag.clone(), seed: rng.next_u64() as u32, by: by.clone(), at: now });
    }
    let idx = r.chambers.len() as u32 - 1;
    let clamp = |c: u32| c.min(r.chambers.len() as u32 - 1);
    let placements: Vec<(u32, Tag)> = plan.features.iter().map(|t| (idx, t.clone())).chain(plan.enrich.iter().map(|(c, t)| (clamp(*c), t.clone()))).collect();
    for (chamber, tag) in placements {
        r.features.push(Feature { id: new_id(), tag, chamber, seed: rng.next_u64() as u32, by: by.clone(), at: now });
    }
    let mut made = Vec::new();
    for (kind, data, chamber) in plan.things {
        let chamber = chamber.map(clamp).unwrap_or(idx);
        let name = data.get("name").or(data.get("title")).and_then(|v| v.as_str()).unwrap_or(&kind).to_string();
        made.push(name);
        r.things.push(Thing { id: new_id(), kind, chamber, seed: rng.next_u64() as u32, data, by: by.clone(), at: now });
    }
    r.attention = (r.attention - threshold).max(0.0);
    r.building = None;
    let what = if new_chamber { plan.chamber_name.clone() } else { "more detail".into() };
    let mut tags = plan.features.clone();
    tags.extend(plan.enrich.iter().map(|(_, t)| t.clone()));
    tags.dedup();
    let mut text = if new_chamber { format!("The architect built {} ({}).", plan.chamber_name, tags.join(", ")) } else { format!("The architect added detail ({}).", tags.join(", ")) };
    if !made.is_empty() {
        text.push_str(&format!(" New here: {}.", made.join(", ")));
    }
    if let Some(v) = &replunged_from {
        text.push_str(&format!(" Brought forward from {v}."));
    }
    push_log(&mut r.log, LogEntry { id: new_id(), kind: "growth".into(), who: "architect".into(), player: None, text: text.clone(), at: now });
    if let Some(n) = plan.note {
        push_log(&mut r.log, LogEntry { id: new_id(), kind: "note".into(), who: "the architect".into(), player: None, text: n, at: now });
    }
    let (x, z) = (r.x, r.z);
    let investors: Vec<String> = r.investors.iter().filter(|(_, s)| **s >= 15.0).map(|(p, _)| p.clone()).collect();
    let summary = room_summary(r);
    let _ = chambers_before;
    if let Some(w) = uni.worlds.get_mut(wid) {
        w.builds_since_tag += 1;
    }
    let mut notes = Vec::new();
    for pid in investors {
        let Some(p) = uni.players.get_mut(&pid) else { continue };
        let n = Notification { id: new_id(), at: now, world: wid.to_string(), x, z, text: format!("{what} grew in {world_name} at {x},{z}."), read: false };
        p.inbox.push(n.clone());
        if p.inbox.len() > 50 {
            p.inbox.remove(0);
        }
        notes.push((pid, json!({ "t": "notify", "n": n })));
    }
    drop(uni);
    app.note_commit(format!("{world_name} {x},{z}: {text} [{by}, {}]", ctx.budget.name()));
    app.touch();
    app.broadcast(wid, &json!({ "t": "room", "addr": key, "summary": summary, "grew": true }));
    for (pid, m) in notes {
        app.send_direct(&pid, &m);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn learns_from_admiration() {
        let mut uni = Universe::default();
        uni.seed_builtins();
        uni.players.insert("p".into(), Player { id: "p".into(), secret: "s".into(), name: "n".into(), color: "#fff".into(), prefs: Default::default(), inbox: vec![], created: 0 });
        let before = desire(&uni.room("the-lush", 0, 0).unwrap(), &uni);
        let tags = record_admire(&mut uni, "the-lush", 0, 0, 0, "p");
        assert!(!tags.is_empty());
        let after = desire(&uni.room("the-lush", 0, 0).unwrap(), &uni);
        let t = &tags[0];
        assert!(after.get(t).unwrap_or(&0.0) >= before.get(t).unwrap_or(&0.0));
        assert!(uni.players["p"].prefs.contains_key(t));
    }

    #[test]
    fn plans_always_build_something() {
        let mut uni = Universe::default();
        uni.seed_builtins();
        let room = uni.room("the-lush", 1, 1).unwrap();
        for budget in [Budget::Sketch, Budget::Normal, Budget::Rich] {
            let ctx = PlanContext {
                world: "the-lush".into(),
                world_name: "The Lush".into(),
                desire: desire(&room, &uni),
                requests: vec!["more snow".into()],
                notes: vec![],
                looked: vec![],
                used: vec![],
                story: Default::default(),
                budget,
                detail: false,
                room: room.clone(),
            };
            for seed in 0..50 {
                let plan = Heuristic.plan(&ctx, seed);
                assert!(!plan.features.is_empty());
                assert!(!plan.chamber_name.is_empty());
                assert!(procgen::TAGS.contains(&plan.chamber_tag.as_str()));
            }
        }
    }

    #[test]
    fn reads_plans_from_chatty_models() {
        let reply = "Sure! Here is the plan:\n```json\n{\"chamber_name\": \"The Salt Library\", \"chamber_tag\": \"books\", \"features\": [\"books\", \"bogus\"], \"things\": [{\"kind\": \"character\", \"data\": {\"name\": \"Ada {curly}\", \"lines\": [\"hi\"]}}]}\n```";
        let p = parse_plan(reply, "test").unwrap();
        assert_eq!(p.chamber_name, "The Salt Library");
        assert_eq!(p.features, vec!["books"]);
        assert_eq!(p.things.len(), 1);
    }
}

//! The architect: an asynchronous builder that grows rooms where people
//! spend their time, biased toward what those people seem to want.
//!
//! `Architect` is the seam other implementations plug into (an LLM-backed
//! architect, a human-in-the-loop one, a host's own style). The reference
//! `Heuristic` architect learns from three signals:
//!   * dwell: seconds spent in a chamber reinforce that chamber's tags,
//!   * admiration: pressing "more of this" strongly reinforces them,
//!   * requests: visitor-log text is mined for tag keywords.
//! It blends the room's own learned weights with the taste profiles of the
//! players who invested time there, then samples with a little novelty.

use crate::procgen::{self, Rng};
use crate::model::*;
use crate::state::{App, room_summary};
use serde_json::json;
use std::collections::BTreeMap;

pub struct PlanContext<'a> {
    pub room: &'a Room,
    /// Blended desire per tag, already normalised to sum 1.
    pub desire: BTreeMap<Tag, f32>,
    pub recent_requests: Vec<&'a str>,
}

pub struct Plan {
    pub chamber_name: String,
    pub chamber_tag: Tag,
    /// Features for the new chamber.
    pub features: Vec<Tag>,
    /// Extra touches added to older chambers.
    pub enrich: Vec<(u32, Tag)>,
}

pub trait Architect: Send + Sync {
    fn id(&self) -> &'static str;
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
    fn id(&self) -> &'static str {
        "reference-heuristic@1"
    }

    fn plan(&self, ctx: &PlanContext, seed: u64) -> Plan {
        let mut rng = Rng::new(seed);
        // A fresh request is honoured half the time; otherwise follow desire.
        let requested: Vec<&str> = ctx.recent_requests.iter().take(2).flat_map(|t| procgen::tags_in_text(t)).collect();
        let chamber_tag = if !requested.is_empty() && rng.f32() < 0.5 {
            rng.pick(&requested).to_string()
        } else {
            sample(&ctx.desire, &mut rng, 0.12)
        };
        let mut features = vec![chamber_tag.clone()];
        let extra = 1 + (rng.next_u64() % 3) as usize;
        for _ in 0..extra {
            features.push(sample(&ctx.desire, &mut rng, 0.2));
        }
        let mut enrich = Vec::new();
        let chambers = ctx.room.chambers.len() as u32;
        if chambers > 1 && rng.f32() < 0.6 {
            let c = (rng.next_u64() % chambers as u64) as u32;
            enrich.push((c, sample(&ctx.desire, &mut rng, 0.1)));
        }
        Plan { chamber_name: procgen::chamber_name(&chamber_tag, &mut rng), chamber_tag, features, enrich }
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

/// One architect step: start builds where attention crossed the threshold
/// and finish builds whose time has come. Runs every second.
pub fn tick(app: &App) {
    let now = now_ms();
    let mut broadcasts = Vec::new();
    let mut directs = Vec::new();
    {
        let mut uni = app.uni.lock().unwrap();
        let world_ids: Vec<String> = uni.worlds.keys().cloned().collect();
        for wid in world_ids {
            let keys: Vec<String> = uni.worlds[&wid].rooms.keys().cloned().collect();
            for key in keys {
                let (growth, attention, building) = {
                    let r = &uni.worlds[&wid].rooms[&key];
                    (r.growth(), r.attention, r.building.clone())
                };
                match building {
                    None if attention >= app.threshold(growth) => {
                        let r = uni.worlds.get_mut(&wid).unwrap().rooms.get_mut(&key).unwrap();
                        let secs = 12 + (procgen::splitmix(r.seed as u64 ^ now) % 30);
                        r.building = Some(Build { started: now, ready_at: now + secs * 1000 });
                        broadcasts.push((wid.clone(), json!({ "t": "room", "addr": key, "summary": room_summary(r) })));
                        app.touch();
                    }
                    Some(b) if now >= b.ready_at => {
                        let (msgs, notes) = finish_build(app, &mut uni, &wid, &key, now);
                        broadcasts.extend(msgs);
                        directs.extend(notes);
                        app.touch();
                    }
                    _ => {}
                }
            }
        }
    }
    for (w, m) in broadcasts {
        app.broadcast(&w, &m);
    }
    for (pid, m) in directs {
        app.send_direct(&pid, &m);
    }
}

type Out = (Vec<(String, serde_json::Value)>, Vec<(String, serde_json::Value)>);

fn finish_build(app: &App, uni: &mut Universe, wid: &str, key: &str, now: u64) -> Out {
    let room = uni.worlds[wid].rooms[key].clone();
    // Only requests made since the last chamber are "fresh".
    let last_built = room.chambers.last().map(|c| c.at).unwrap_or(0);
    let ctx = PlanContext {
        room: &room,
        desire: desire(&room, uni),
        recent_requests: room.log.iter().rev().filter(|l| l.kind == "request" && l.at > last_built).take(5).map(|l| l.text.as_str()).collect(),
    };
    let seed = procgen::splitmix(room.seed as u64 ^ (room.chambers.len() as u64) << 32 ^ now);
    let plan = app.architect.plan(&ctx, seed);
    let world_name = uni.worlds[wid].manifest.name.clone();
    let threshold = app.threshold(room.growth());

    let r = uni.worlds.get_mut(wid).unwrap().rooms.get_mut(key).unwrap();
    let idx = r.chambers.len() as u32;
    let by = app.architect.id().to_string();
    let mut rng = Rng::new(seed);
    r.chambers.push(Chamber {
        name: plan.chamber_name.clone(),
        tag: plan.chamber_tag.clone(),
        seed: rng.next_u64() as u32,
        by: by.clone(),
        at: now,
    });
    let placements = plan.features.iter().map(|t| (idx, t.clone())).chain(plan.enrich.iter().cloned());
    for (chamber, tag) in placements {
        r.features.push(Feature { id: new_id(), tag, chamber, seed: rng.next_u64() as u32, by: by.clone(), at: now });
    }
    r.attention = (r.attention - threshold).max(0.0);
    r.building = None;
    let mut tags = plan.features.clone();
    tags.dedup();
    let text = format!("The architect built {} ({}).", plan.chamber_name, tags.join(", "));
    push_log(&mut r.log, LogEntry { id: new_id(), kind: "growth".into(), who: "architect".into(), player: None, text, at: now });

    let (x, z) = (r.x, r.z);
    let investors: Vec<String> = r.investors.iter().filter(|(_, s)| **s >= 15.0).map(|(p, _)| p.clone()).collect();
    let summary = room_summary(r);

    let mut notes = Vec::new();
    for pid in investors {
        let Some(p) = uni.players.get_mut(&pid) else { continue };
        let n = Notification {
            id: new_id(),
            at: now,
            world: wid.to_string(),
            x,
            z,
            text: format!("{} grew in {} at {},{}.", plan.chamber_name, world_name, x, z),
            read: false,
        };
        p.inbox.push(n.clone());
        if p.inbox.len() > 50 {
            p.inbox.remove(0);
        }
        notes.push((pid, json!({ "t": "notify", "n": n })));
    }
    (vec![(wid.to_string(), json!({ "t": "room", "addr": key, "summary": summary, "grew": true }))], notes)
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
        let ctx = PlanContext { room: &room, desire: desire(&room, &uni), recent_requests: vec!["more snow"] };
        for seed in 0..50 {
            let plan = Heuristic.plan(&ctx, seed);
            assert!(!plan.features.is_empty());
            assert!(procgen::TAGS.contains(&plan.chamber_tag.as_str()));
        }
    }
}

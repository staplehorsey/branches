//! New worlds and evolving biomes.
//!
//! Wandering grows the graph. Every time a world's explorers have walked
//! onto enough fresh ground, a new world is born from the shape of their
//! paths (winding or straight, hurried or lingering, revisiting or always
//! new) blended with the world they were walking in. A door pair joins the
//! house they were standing by to the new world's first house.
//!
//! Biomes evolve too: each tagged version nudges a world's look, steered by
//! how people use it, so the far, older bands of a world show how it was.

use crate::model::*;
use crate::procgen::{Rng, splitmix};
use crate::state::App;
use serde_json::{Value, json};

/// Fresh cells walked before a new world is born.
pub const BIRTH_CELLS: u32 = 36;
const MAX_WORLDS: usize = 64;

#[derive(Clone, Copy, Debug, Default)]
pub struct Sig {
    /// Radians of turning per metre walked.
    pub turn: f32,
    /// Displacement over distance, 0 (circling) to 1 (a straight line).
    pub straight: f32,
    /// Metres per second while moving.
    pub speed: f32,
    /// Share of time spent on ground already walked.
    pub revisit: f32,
    /// Average heading, radians.
    pub heading: f32,
}

fn num(v: &Value, k: &str) -> f32 {
    v.get(k).and_then(|x| x.as_f64()).unwrap_or(0.0) as f32
}

/// The recent paths of everyone in a world, blended into one shape.
pub fn blend(paths: &[Value]) -> Sig {
    let recent: Vec<&Value> = paths.iter().rev().take(10).collect();
    if recent.is_empty() {
        return Sig { straight: 0.5, speed: 3.0, ..Default::default() };
    }
    let n = recent.len() as f32;
    let (mut s, mut hx, mut hz) = (Sig::default(), 0.0f32, 0.0f32);
    for p in recent {
        s.turn += num(p, "turn") / n;
        s.straight += num(p, "straight") / n;
        s.speed += num(p, "speed") / n;
        s.revisit += num(p, "revisit") / n;
        hx += num(p, "heading").cos();
        hz += num(p, "heading").sin();
    }
    s.heading = hz.atan2(hx);
    s
}

// ------------------------------------------------------------ colour

fn hex_to_hsl(hex: &str) -> Option<(f32, f32, f32)> {
    let h = hex.trim_start_matches('#');
    if h.len() != 6 {
        return None;
    }
    let p = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).ok().map(|v| v as f32 / 255.0);
    let (r, g, b) = (p(0)?, p(2)?, p(4)?);
    let (max, min) = (r.max(g).max(b), r.min(g).min(b));
    let l = (max + min) / 2.0;
    if (max - min).abs() < 1e-6 {
        return Some((0.0, 0.0, l));
    }
    let d = max - min;
    let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
    let hue = if max == r { (g - b) / d + if g < b { 6.0 } else { 0.0 } } else if max == g { (b - r) / d + 2.0 } else { (r - g) / d + 4.0 };
    Some((hue / 6.0, s, l))
}

fn hsl_to_hex(h: f32, s: f32, l: f32) -> String {
    let f = |n: f32| {
        let k = (n + h * 12.0) % 12.0;
        let a = s * l.min(1.0 - l);
        let c = l - a * (k - 3.0).min(9.0 - k).clamp(-1.0, 1.0);
        (c.clamp(0.0, 1.0) * 255.0).round() as u8
    };
    format!("#{:02x}{:02x}{:02x}", f(0.0), f(8.0), f(4.0))
}

/// Shift every colour in a biome: hue by `dh` turns, saturation and
/// lightness by small amounts.
fn recolor(v: &mut Value, dh: f32, ds: f32, dl: f32) {
    match v {
        Value::String(s) if s.starts_with('#') && s.len() == 7 => {
            if let Some((h, sa, l)) = hex_to_hsl(s) {
                *s = hsl_to_hex((h + dh).rem_euclid(1.0), (sa + ds).clamp(0.0, 1.0), (l + dl).clamp(0.03, 0.97));
            }
        }
        Value::Array(a) => a.iter_mut().for_each(|x| recolor(x, dh, ds, dl)),
        Value::Object(o) => o.values_mut().for_each(|x| recolor(x, dh, ds, dl)),
        _ => {}
    }
}

fn scale(v: &mut Value, path: &[&str], by: f32, lo: f64, hi: f64) {
    let mut cur = v;
    for k in &path[..path.len() - 1] {
        let Some(next) = cur.get_mut(*k) else { return };
        cur = next;
    }
    if let Some(x) = cur.get_mut(path[path.len() - 1]) {
        if let Some(n) = x.as_f64() {
            *x = json!((n * by as f64).clamp(lo, hi));
        }
    }
}

const TREES: &[&str] = &["round", "pine", "blossom", "palm", "mushroom"];
const PLACES: &[&str] = &["house", "tower", "cave", "arch"];

/// Mutate a biome. `amount` is 0..1; `sig` steers it when given.
pub fn mutate(params: &Value, rng: &mut Rng, amount: f32, sig: Option<Sig>) -> Value {
    let mut b = params.clone();
    let swing = |rng: &mut Rng| (rng.f32() - 0.5) * 2.0;
    let dh = match sig {
        // Where people head becomes the colour of the place they find.
        Some(s) => s.heading / std::f32::consts::TAU * 0.6 * amount + swing(rng) * 0.04,
        None => swing(rng) * 0.06 * amount,
    };
    recolor(&mut b, dh, swing(rng) * 0.08 * amount, swing(rng) * 0.05 * amount);
    let s = sig.unwrap_or(Sig { straight: 0.5, speed: 3.0, ..Default::default() });
    // Lingering and circling make close, foggy, overgrown places; hurried
    // straight lines make open, sparse, wide ones.
    let close = (s.revisit + (1.0 - s.straight)) * 0.5;
    let fog = 1.0 + (0.5 - close) * 0.6 * amount + swing(rng) * 0.1 * amount;
    scale(&mut b, &["fog", "near"], fog, 3.0, 60.0);
    scale(&mut b, &["fog", "far"], fog, 40.0, 240.0);
    scale(&mut b, &["grass", "density"], 1.0 + (close - 0.5) * 0.6 * amount, 0.1, 1.4);
    scale(&mut b, &["grass", "height"], 1.0 + swing(rng) * 0.15 * amount, 0.2, 0.9);
    scale(&mut b, &["terrain", "amp"], 1.0 + (s.turn.min(1.0) - 0.3) * amount + swing(rng) * 0.2 * amount, 0.2, 10.0);
    scale(&mut b, &["terrain", "scale"], 1.0 + swing(rng) * 0.2 * amount, 25.0, 140.0);
    scale(&mut b, &["ponds"], 1.0 + swing(rng) * 0.3 * amount, 0.0, 0.8);
    scale(&mut b, &["sun", "elevation"], 1.0 + swing(rng) * 0.2 * amount, 0.08, 1.4);
    if rng.f32() < 0.25 * amount {
        b["trees"] = json!(*rng.pick(TREES));
    }
    // The mix of places doors live in drifts too.
    if let Some(places) = b.get_mut("places").and_then(|p| p.as_object_mut()) {
        for k in PLACES {
            let w = places.get(*k).and_then(|v| v.as_f64()).unwrap_or(0.1) as f32;
            places.insert((*k).into(), json!((w * (1.0 + swing(rng) * 0.5 * amount)).clamp(0.02, 1.0)));
        }
    }
    b
}

const NOUNS_STRAIGHT: &[&str] = &["Mile", "Avenue", "Causeway", "Long Field", "Meridian"];
const NOUNS_WINDING: &[&str] = &["Meander", "Coil", "Glade", "Spiral", "Labyrinth"];
const ADJ_SLOW: &[&str] = &["Hushed", "Drowsy", "Lingering", "Patient", "Velvet"];
const ADJ_FAST: &[&str] = &["Bright", "Running", "Restless", "Quick", "Windward"];

/// Maybe a new world is born where someone just walked onto fresh ground.
pub fn maybe_birth(app: &App, uni: &mut Universe, world: &str, cell: (i32, i32), player: &str) -> Option<Value> {
    let w = uni.worlds.get_mut(world)?;
    if w.fresh_cells < BIRTH_CELLS {
        return None;
    }
    w.fresh_cells = 0;
    if uni.worlds.len() >= MAX_WORLDS {
        return None;
    }
    let w = &uni.worlds[world];
    let sig = blend(&w.paths);
    let seed = splitmix(w.manifest.seed as u64 ^ ((cell.0 as i64 as u64) << 20) ^ (cell.1 as i64 as u64) ^ crate::model::now_ms());
    let mut rng = Rng::new(seed);
    let adj = if sig.speed > 4.5 { *rng.pick(ADJ_FAST) } else { *rng.pick(ADJ_SLOW) };
    let noun = if sig.straight > 0.6 { *rng.pick(NOUNS_STRAIGHT) } else { *rng.pick(NOUNS_WINDING) };
    let name = format!("The {adj} {noun}");
    let id = format!("{}-{:04x}", name.to_lowercase().trim_start_matches("the ").replace(' ', "-"), seed & 0xffff);
    let biome = mutate(&w.manifest.generator.params, &mut rng, 1.0, Some(sig));
    let manifest = WorldManifest {
        id: id.clone(),
        name: name.clone(),
        tagline: format!("Born from paths walked in {}: {}.", w.manifest.name, describe(sig)),
        seed: (seed >> 32) as u32,
        generator: Generator { kind: "liminal-houses@1".into(), params: biome },
        portal_policy: PortalPolicy { mode: PolicyMode::Open, max_per_room: 3, allow_hosts: vec![], allow_local: true },
        architect: "reference-heuristic@2".into(),
        spawn: "0,0".into(),
        hub: false,
        parent: Some(world.to_string()),
        signature: Some(json!({ "turn": sig.turn, "straight": sig.straight, "speed": sig.speed, "revisit": sig.revisit, "heading": sig.heading })),
    };
    let parent_name = w.manifest.name.clone();
    uni.worlds.insert(id.clone(), World::new(manifest));
    let now = now_ms();
    let who = uni.players.get(player).map(|p| p.name.clone()).unwrap_or_else(|| "someone".into());
    // The door out, in the house they were walking past...
    let here = uni.room_mut(world, cell.0, cell.1)?;
    let slot = if here.portals.iter().filter(|p| p.slot == 0).count() < 6 { 0 } else { here.growth() };
    let door = Portal { id: format!("born-{id}"), slot, target: format!("/w/{id}/0,0"), label: name.clone(), by: "genesis".into(), at: now, sealed: false };
    here.portals.push(door);
    push_log(&mut here.log, LogEntry { id: new_id(), kind: "portal".into(), who: who.clone(), player: Some(player.into()), text: format!("walked a world into being: {name}"), at: now });
    // ...and the door back, in the new world's first house.
    let first = uni.room_mut(&id, 0, 0)?;
    first.portals.push(Portal { id: format!("born-{id}-back"), slot: 0, target: format!("/w/{world}/{},{}", cell.0, cell.1), label: parent_name, by: "genesis".into(), at: now, sealed: false });
    app.note_commit(format!("New world {name} ({id}), born from paths in {world}"));
    app.broadcast(world, &json!({ "t": "room", "addr": addr(cell.0, cell.1), "rebuild": true }));
    Some(json!({ "t": "born", "world": id, "name": name, "addr": addr(cell.0, cell.1), "about": describe(sig) }))
}

fn describe(s: Sig) -> String {
    let way = if s.straight > 0.6 { "straight lines" } else if s.turn > 0.4 { "winding turns" } else { "meandering" };
    let pace = if s.speed > 4.5 { "in a hurry" } else { "taking their time" };
    let back = if s.revisit > 0.4 { ", returning to the same streets" } else { "" };
    format!("{way}, {pace}{back}")
}

/// Before a world is tagged, its biome drifts a little, steered by how
/// people have been using it.
pub fn evolve(w: &mut World, seed: u64) {
    let mut rng = Rng::new(seed);
    let sig = if w.paths.is_empty() { None } else { Some(blend(&w.paths)) };
    w.manifest.generator.params = mutate(&w.manifest.generator.params, &mut rng, 0.35, sig);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colours_round_trip() {
        for c in ["#6fb9ec", "#ffffff", "#000000", "#e8a697"] {
            let (h, s, l) = hex_to_hsl(c).unwrap();
            assert_eq!(hsl_to_hex(h, s, l), c);
        }
    }

    #[test]
    fn mutation_keeps_biomes_valid() {
        let base = crate::procgen::builtin_worlds()[0].generator.params.clone();
        let mut rng = Rng::new(7);
        let m = mutate(&base, &mut rng, 1.0, Some(Sig { turn: 0.8, straight: 0.2, speed: 2.0, revisit: 0.6, heading: 1.0 }));
        assert!(m["sky"]["top"].as_str().unwrap().starts_with('#'));
        assert!(m["fog"]["near"].as_f64().unwrap() >= 3.0);
        assert_ne!(m["sky"]["top"], base["sky"]["top"]);
    }
}

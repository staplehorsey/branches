//! The atlas: a map of the ideas the dream has explored, so that the next
//! one is chosen by structure, not by asking a model to "be creative" (which
//! keeps landing in the same place).
//!
//! Every space is a node with coordinates on a few explicit axes (where the
//! idea comes from, the feeling at its core, what you do there, how its
//! story is shaped, its scale, era, register and medium). The next node is
//! picked by code:
//!   * a new house goes somewhere far from everything explored, pulled
//!     toward what you marked warmer and pushed from what you marked colder;
//!   * the next space in a house is a step from the last: one axis changes
//!     when you've been saying warmer, two by default, three or four when
//!     you've been saying colder, never the axis changed last time.
//! Claude then has to synthesise a concrete idea at those coordinates, shown
//! the nearest explored nodes (to be clearly unlike) and the motifs used too
//! often (to be avoided). Spaces report their motifs; the ledger grows.
//!
//! Lives in `.branches/atlas.json` with your worlds; it's plain JSON so a
//! model (or you) can read and extend it, including the axes themselves.

use crate::model::now_ms;
use crate::procgen::Rng;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub type Genome = BTreeMap<String, String>;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Node {
    pub id: String,
    pub genome: Genome,
    pub name: String,
    pub premise: String,
    #[serde(default)]
    pub motifs: Vec<String>,
    pub world: String,
    pub x: i32,
    pub z: i32,
    pub space: u32,
    /// The node it stepped from (the previous space in the house).
    #[serde(default)]
    pub from: Option<String>,
    /// Warmer minus colder, and seconds spent.
    #[serde(default)]
    pub score: i32,
    #[serde(default)]
    pub dwell: f64,
    pub at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Atlas {
    pub axes: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub nodes: Vec<Node>,
}

fn axes() -> BTreeMap<String, Vec<String>> {
    let v = |l: &[&str]| l.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    BTreeMap::from([
        ("source".into(), v(&["one of the dreamer's muses", "one of the dreamer's muses", "a novel", "a song", "a film", "a myth or folk tale", "a scientific idea", "a historical event", "a craft or trade", "a sport or game", "food and cooking", "a religious practice", "law and bureaucracy", "money and markets", "medicine and the body", "a machine or technology", "an animal's life", "weather and geology", "a childhood ritual", "a family argument", "a city's infrastructure", "a painting", "a mathematical object", "a job nobody notices"])),
        ("feeling".into(), v(&["grief", "awe", "desire", "shame", "joy", "dread", "tenderness", "futility", "rage", "nostalgia", "curiosity", "jealousy", "relief", "guilt", "pride", "loneliness", "absurdity", "devotion", "boredom made beautiful", "suspicion"])),
        ("verb".into(), v(&["repair", "sort and classify", "cook", "trade and barter", "investigate", "tend and grow", "build", "navigate", "perform", "translate and decode", "remember and reconstruct", "hide and seek", "deliver", "listen", "measure", "choose and sacrifice", "clean and restore", "assemble a machine", "care for someone", "escape"])),
        ("structure".into(), v(&["mystery with clues", "journey with stations", "ritual in steps", "cycle that changes each turn", "transformation", "countdown", "letters and documents", "a choice with consequences", "a collection to complete", "unravelling a lie", "a negotiation", "a day in a life", "a heist", "an apprenticeship", "a reunion"])),
        ("scale".into(), v(&["hand-held", "a single room", "a building", "a street", "a landscape", "an ocean", "a city", "the inside of a body", "a planet", "a dollhouse"])),
        ("era".into(), v(&["prehistoric", "ancient", "medieval", "early modern", "industrial", "1950s", "1980s", "the present", "near future", "far future", "mythic time", "a child's afternoon"])),
        ("register".into(), v(&["comic", "austere", "baroque", "clinical", "warm and domestic", "eerie", "epic", "intimate", "noisy and crowded", "festive", "bureaucratic", "pastoral", "industrial grit", "lush and saturated"])),
        ("medium".into(), v(&["first-person 3D", "top-down 2D", "side-on 2D", "a diorama you orbit", "text and choices", "sound-led, eyes half closed", "a desk of objects and papers", "a table game", "a vehicle you drive", "a sequence of rooms you edit"])),
    ])
}

impl Default for Atlas {
    fn default() -> Self {
        Atlas { axes: axes(), nodes: Vec::new() }
    }
}

static LOCK: Mutex<()> = Mutex::new(());

fn file(root: &Path) -> PathBuf {
    root.join(".branches/atlas.json")
}

pub fn load(root: &Path) -> Atlas {
    let mut a: Atlas = std::fs::read(file(root)).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default();
    // New axes added in code reach old atlases; values you added stay.
    for (k, v) in axes() {
        a.axes.entry(k).or_insert(v);
    }
    a
}

pub fn update<T>(root: &Path, f: impl FnOnce(&mut Atlas) -> T) -> T {
    let _g = LOCK.lock().unwrap();
    let mut a = load(root);
    let out = f(&mut a);
    let _ = std::fs::create_dir_all(root.join(".branches"));
    let _ = std::fs::write(file(root), serde_json::to_vec_pretty(&a).unwrap_or_default());
    out
}

/// Share of axes on which two genomes differ (0 same, 1 entirely different).
pub fn distance(a: &Genome, b: &Genome) -> f32 {
    let keys: Vec<&String> = a.keys().chain(b.keys()).collect::<std::collections::BTreeSet<_>>().into_iter().collect();
    if keys.is_empty() {
        return 1.0;
    }
    keys.iter().filter(|k| a.get(**k) != b.get(**k)).count() as f32 / keys.len() as f32
}

fn random_genome(axes: &BTreeMap<String, Vec<String>>, rng: &mut Rng) -> Genome {
    axes.iter().filter(|(_, v)| !v.is_empty()).map(|(k, v)| (k.clone(), rng.pick(v).clone())).collect()
}

/// How a candidate scores: far from everything (novelty), near what was
/// warm, away from what was cold.
fn score(g: &Genome, nodes: &[Node]) -> f32 {
    if nodes.is_empty() {
        return 1.0;
    }
    let novelty = nodes.iter().map(|n| distance(g, &n.genome)).fold(1.0f32, f32::min);
    let mut pull = 0.0;
    for n in nodes.iter().filter(|n| n.score != 0) {
        let near = 1.0 - distance(g, &n.genome);
        pull += near * near * (n.score.clamp(-3, 3) as f32) / 3.0;
    }
    novelty + 0.35 * pull
}

pub struct Choice {
    pub genome: Genome,
    pub from: Option<Node>,
    /// The axes that changed from the last space, and why.
    pub why: String,
    pub nearest: Vec<Node>,
    pub overused: Vec<String>,
}

/// Where the next space should be: a step from the house's last node, or
/// for a house's first space, somewhere new.
pub fn choose(root: &Path, world: &str, x: i32, z: i32, recent_feel: i32, seed: u64) -> Choice {
    let a = load(root);
    let mut rng = Rng::new(seed);
    let prev = a.nodes.iter().filter(|n| n.world == world && n.x == x && n.z == z).max_by_key(|n| n.space).cloned();
    let (genome, why) = match &prev {
        Some(p) => {
            let k = match recent_feel {
                f if f >= 2 => 1,
                f if f <= -2 => 3 + (rng.next_u64() % 2) as usize,
                _ => 2,
            };
            // Don't turn the same dial twice in a row.
            let last_changed: Vec<String> = p.from.as_ref().and_then(|id| a.nodes.iter().find(|n| &n.id == id)).map(|pp| p.genome.iter().filter(|(k, v)| pp.genome.get(*k) != Some(v)).map(|(k, _)| k.clone()).collect()).unwrap_or_default();
            let mut best: Option<(f32, Genome, Vec<String>)> = None;
            for _ in 0..120 {
                let mut g = p.genome.clone();
                let mut keys: Vec<String> = a.axes.keys().filter(|k| !last_changed.contains(k)).cloned().collect();
                if keys.len() < k {
                    keys = a.axes.keys().cloned().collect();
                }
                let mut changed = Vec::new();
                for _ in 0..k {
                    if keys.is_empty() {
                        break;
                    }
                    let key = keys.remove((rng.next_u64() % keys.len() as u64) as usize);
                    let vals: Vec<&String> = a.axes[&key].iter().filter(|v| g.get(&key) != Some(*v)).collect();
                    if let Some(v) = vals.get((rng.next_u64() % vals.len().max(1) as u64) as usize) {
                        g.insert(key.clone(), (*v).clone());
                        changed.push(key);
                    }
                }
                let s = score(&g, &a.nodes) + rng.f32() * 0.05;
                if best.as_ref().is_none_or(|b| s > b.0) {
                    best = Some((s, g, changed));
                }
            }
            let (_, g, changed) = best.unwrap_or((0.0, p.genome.clone(), vec![]));
            let mood = match recent_feel {
                f if f >= 2 => "they've been saying warmer: one small step",
                f if f <= -2 => "they've been saying colder: a big step",
                _ => "a step",
            };
            let why = format!("{mood}; changed {}", changed.iter().map(|k| format!("{k}: \"{}\" → \"{}\"", p.genome.get(k).cloned().unwrap_or_default(), g.get(k).cloned().unwrap_or_default())).collect::<Vec<_>>().join(", "));
            (g, why)
        }
        None => {
            let mut best: Option<(f32, Genome)> = None;
            for _ in 0..300 {
                let g = random_genome(&a.axes, &mut rng);
                let s = score(&g, &a.nodes) + rng.f32() * 0.05;
                if best.as_ref().is_none_or(|b| s > b.0) {
                    best = Some((s, g));
                }
            }
            (best.map(|b| b.1).unwrap_or_default(), "a new house: as far as possible from everything explored, leaning toward what was warm".into())
        }
    };
    let mut nearest: Vec<Node> = a.nodes.clone();
    nearest.sort_by(|p, q| distance(&genome, &p.genome).total_cmp(&distance(&genome, &q.genome)));
    nearest.truncate(5);
    Choice { genome, from: prev, why, nearest, overused: overused(&a, 15) }
}

/// Motifs used in more than one space, most used first.
pub fn overused(a: &Atlas, n: usize) -> Vec<String> {
    let mut count: BTreeMap<String, usize> = BTreeMap::new();
    for node in &a.nodes {
        for m in &node.motifs {
            *count.entry(m.trim().to_lowercase()).or_default() += 1;
        }
    }
    let mut v: Vec<(String, usize)> = count.into_iter().filter(|(_, c)| *c >= 2).collect();
    v.sort_by(|p, q| q.1.cmp(&p.1));
    v.into_iter().take(n).map(|(m, c)| format!("{m} (×{c})")).collect()
}

pub fn add(root: &Path, mut node: Node) {
    node.at = now_ms();
    if node.id.is_empty() {
        node.id = crate::model::new_id();
    }
    update(root, |a| {
        a.nodes.retain(|n| !(n.world == node.world && n.x == node.x && n.z == node.z && n.space == node.space));
        a.nodes.push(node);
    });
}

/// New values a space proposed for the axes (up to two per axis per space,
/// at most 60 per axis), so the space being explored can widen.
pub fn extend(root: &Path, proposed: &serde_json::Value) {
    let Some(obj) = proposed.as_object() else { return };
    update(root, |a| {
        for (axis, vals) in obj {
            let Some(list) = a.axes.get_mut(axis) else { continue };
            for v in vals.as_array().into_iter().flatten().filter_map(|v| v.as_str()).take(2) {
                let v: String = v.trim().chars().take(60).collect();
                if !v.is_empty() && list.len() < 60 && !list.iter().any(|x| x.eq_ignore_ascii_case(&v)) {
                    list.push(v);
                }
            }
        }
    });
}

/// Warmer or colder on a space: the node remembers.
pub fn feel(root: &Path, world: &str, x: i32, z: i32, space: u32, score: i32) {
    update(root, |a| {
        if let Some(n) = a.nodes.iter_mut().find(|n| n.world == world && n.x == x && n.z == z && n.space == space) {
            n.score += score;
        }
    });
}

pub fn dwell(root: &Path, world: &str, x: i32, z: i32, space: u32, secs: f64) {
    update(root, |a| {
        if let Some(n) = a.nodes.iter_mut().find(|n| n.world == world && n.x == x && n.z == z && n.space == space) {
            n.dwell += secs;
        }
    });
}

pub fn describe(g: &Genome) -> String {
    g.iter().map(|(k, v)| format!("  {k}: {v}")).collect::<Vec<_>>().join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explores_away_and_steps_near() {
        let dir = std::env::temp_dir().join(format!("atlas-test-{}", now_ms()));
        std::fs::create_dir_all(&dir).unwrap();
        let c1 = choose(&dir, "w", 0, 0, 0, 1);
        add(&dir, Node { genome: c1.genome.clone(), world: "w".into(), x: 0, z: 0, space: 1, name: "one".into(), ..Default::default() });
        // A new house lands far from the first.
        let c2 = choose(&dir, "w", 5, 5, 0, 2);
        assert!(distance(&c1.genome, &c2.genome) > 0.6, "{}", distance(&c1.genome, &c2.genome));
        // The next space in the same house is a step: two axes by default, one when warm.
        let step = choose(&dir, "w", 0, 0, 0, 3);
        assert_eq!((distance(&c1.genome, &step.genome) * c1.genome.len() as f32).round() as usize, 2);
        let warm = choose(&dir, "w", 0, 0, 3, 4);
        assert_eq!((distance(&c1.genome, &warm.genome) * c1.genome.len() as f32).round() as usize, 1);
        add(&dir, Node { motifs: vec!["Doors".into()], world: "w".into(), space: 2, ..Default::default() });
        add(&dir, Node { motifs: vec!["doors ".into()], world: "w".into(), space: 3, ..Default::default() });
        assert_eq!(overused(&load(&dir), 5), vec!["doors (×2)"]);
        let _ = std::fs::remove_dir_all(&dir);
    }
}

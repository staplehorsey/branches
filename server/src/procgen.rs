//! Deterministic generation: hashing, the theme/tag vocabulary, the built-in
//! starter worlds and the synthesis of rooms nobody has touched yet.

use crate::model::*;
use serde_json::json;
use std::collections::{BTreeMap, HashMap};

pub fn splitmix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

pub fn hash(seed: u32, x: i32, z: i32, salt: u64) -> u64 {
    splitmix(seed as u64 ^ splitmix((x as i64 as u64) ^ splitmix((z as i64 as u64) ^ splitmix(salt))))
}

/// Small deterministic RNG for planning.
pub struct Rng(u64);
impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(splitmix(seed))
    }
    pub fn next_u64(&mut self) -> u64 {
        self.0 = splitmix(self.0);
        self.0
    }
    pub fn f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }
    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[(self.next_u64() % items.len() as u64) as usize]
    }
}

pub const TAGS: &[&str] = &[
    "water", "plants", "light", "books", "art", "cozy", "sky", "creatures", "music", "stone", "snow",
    "neon",
];

pub struct Theme {
    pub id: &'static str,
    pub name: &'static str,
    pub tags: &'static [&'static str],
}

pub const THEMES: &[Theme] = &[
    Theme { id: "poolrooms", name: "Poolrooms", tags: &["water", "light", "stone"] },
    Theme { id: "moss-library", name: "Moss Library", tags: &["books", "plants", "cozy"] },
    Theme { id: "cloud-nursery", name: "Cloud Nursery", tags: &["sky", "cozy", "light"] },
    Theme { id: "sunset-terrarium", name: "Sunset Terrarium", tags: &["plants", "light", "stone"] },
    Theme { id: "night-aquarium", name: "Night Aquarium", tags: &["water", "creatures", "neon"] },
    Theme { id: "vapor-mall", name: "Vapor Mall", tags: &["neon", "plants", "art"] },
    Theme { id: "tea-garden", name: "Tea Garden", tags: &["water", "plants", "stone"] },
    Theme { id: "fern-cathedral", name: "Fern Cathedral", tags: &["plants", "stone", "light"] },
    Theme { id: "arcade-after-hours", name: "Arcade After Hours", tags: &["neon", "music", "light"] },
    Theme { id: "snowglobe-den", name: "Snowglobe Den", tags: &["snow", "cozy", "light"] },
    Theme { id: "citrus-kitchen", name: "Citrus Kitchen", tags: &["plants", "light", "cozy"] },
    Theme { id: "velvet-theatre", name: "Velvet Theatre", tags: &["music", "art", "cozy"] },
];

pub fn theme(id: &str) -> Option<&'static Theme> {
    THEMES.iter().find(|t| t.id == id)
}

/// Words in visitor requests that hint at a tag.
const KEYWORDS: &[(&str, &[&str])] = &[
    ("water", &["water", "pool", "fountain", "ocean", "swim", "rain", "river", "lake", "bath", "wet", "sea"]),
    ("plants", &["plant", "garden", "green", "moss", "fern", "tree", "flower", "jungle", "lush", "vine", "leaf", "leaves", "bloom"]),
    ("light", &["light", "bright", "glow", "sun", "lamp", "candle", "window", "golden"]),
    ("books", &["book", "library", "read", "poem", "story", "shelf", "shelves", "archive"]),
    ("art", &["art", "painting", "sculpture", "statue", "gallery", "mural", "portrait", "frame"]),
    ("cozy", &["cozy", "cosy", "sofa", "couch", "bed", "pillow", "warm", "blanket", "chair", "sit", "rest", "nap"]),
    ("sky", &["sky", "cloud", "star", "moon", "skylight", "air", "float", "heaven"]),
    ("creatures", &["fish", "jelly", "bird", "butterfl", "cat", "creature", "animal", "firefl", "koi", "moth"]),
    ("music", &["music", "song", "piano", "sound", "record", "radio", "dance", "sing", "chime"]),
    ("stone", &["stone", "column", "marble", "arch", "temple", "ruin", "stair", "pillar", "statue"]),
    ("snow", &["snow", "ice", "winter", "cold", "frost", "crystal"]),
    ("neon", &["neon", "arcade", "synth", "vapor", "retro", "pink", "cyber", "disco"]),
];

pub fn tags_in_text(text: &str) -> Vec<&'static str> {
    let lower = text.to_lowercase();
    KEYWORDS
        .iter()
        .filter(|(_, words)| words.iter().any(|w| lower.contains(w)))
        .map(|(tag, _)| *tag)
        .collect()
}

const ADJECTIVES: &[&str] = &[
    "Quiet", "Drowsy", "Endless", "Soft", "Hollow", "Golden", "Sunken", "Lantern", "Velvet", "Lucid",
    "Gentle", "Forgotten", "Humming", "Pale", "Tender", "Still", "Second", "Inner", "Slow", "Lower",
];

fn nouns(tag: &str) -> &'static [&'static str] {
    match tag {
        "water" => &["Cistern", "Baths", "Tide Room", "Wading Hall", "Lagoon"],
        "plants" => &["Conservatory", "Greenhouse", "Fernery", "Arbor", "Hothouse"],
        "light" => &["Atrium", "Sunroom", "Lantern Hall", "Solarium"],
        "books" => &["Reading Room", "Archive", "Stacks", "Study"],
        "art" => &["Gallery", "Studio", "Salon"],
        "cozy" => &["Parlor", "Den", "Nook", "Lounge"],
        "sky" => &["Observatory", "Cloud Loft", "Skyroom"],
        "creatures" => &["Menagerie", "Aviary", "Aquarium"],
        "music" => &["Music Room", "Listening Room", "Ballroom"],
        "stone" => &["Colonnade", "Cloister", "Rotunda"],
        "snow" => &["Frost Hall", "Winter Room", "Ice House"],
        "neon" => &["Arcade", "Afterglow", "Night Market"],
        _ => &["Room"],
    }
}

pub fn chamber_name(tag: &str, rng: &mut Rng) -> String {
    format!("The {} {}", rng.pick(ADJECTIVES), rng.pick(nouns(tag)))
}

fn manifest(
    id: &str,
    name: &str,
    tagline: &str,
    seed: u32,
    policy: PortalPolicy,
    params: serde_json::Value,
) -> WorldManifest {
    WorldManifest {
        id: id.into(),
        name: name.into(),
        tagline: tagline.into(),
        seed,
        generator: Generator { kind: "liminal-houses@1".into(), params },
        portal_policy: policy,
        architect: "reference-heuristic@1".into(),
        spawn: "0,0".into(),
        hub: true,
        parent: None,
        signature: None,
    }
}

fn open(max: u32) -> PortalPolicy {
    PortalPolicy { mode: PolicyMode::Open, max_per_room: max, allow_hosts: vec![], allow_local: true }
}

/// The starter constellation. Same generator, five very different moods.
pub fn builtin_worlds() -> Vec<WorldManifest> {
    vec![
        manifest(
            "the-lush",
            "The Lush",
            "An endless suburb, overgrown and kind. Every house is the same until you go inside.",
            0x1d5a_17e1,
            open(4),
            json!({
                "sky": { "top": "#6fb9ec", "horizon": "#f3f0dc", "bottom": "#dfeedd" },
                "fog": { "color": "#dcebd9", "near": 30, "far": 150 },
                "sun": { "color": "#fff1d2", "intensity": 2.4, "elevation": 0.9, "azimuth": 0.6 },
                "hemi": { "sky": "#cfe9ff", "ground": "#4f7a3c", "intensity": 1.1 },
                "ground": "#4a8a3c",
                "grass": { "base": "#2f7a32", "tip": "#b8e46c", "density": 1.0, "height": 0.55 },
                "path": "#efe6d2",
                "house": { "wall": "#e7efe4", "roof": "#86b3a2", "trim": "#ffffff", "door": "#f4b9c1", "window": "#fff0c2" },
                "trees": "round",
                "foliage": ["#3e9b4f", "#5fbf5a", "#2f7d45", "#78c95e"],
                "trunk": "#7a5a43",
                "flowers": ["#ff8fb1", "#ffd36e", "#ffffff", "#b78cff", "#ff9e6b"],
                "hedge": "#2f7a3d",
                "lamp": "#fff0c0",
                "night": false,
                "motes": "#fffbe0",
                "water": "#79d3cc",
                "ponds": 0.25,
                "terrain": { "amp": 2.6, "scale": 64 },
                "places": { "house": 0.7, "tower": 0.1, "cave": 0.08, "arch": 0.12 }
            }),
        ),
        manifest(
            "dusk-orchard",
            "Dusk Orchard",
            "Blossom streets at the golden minute before evening that never ends.",
            0x0dc5_0a7d,
            open(3),
            json!({
                "sky": { "top": "#4c3f86", "horizon": "#ffb08a", "bottom": "#f2a596" },
                "fog": { "color": "#e8a697", "near": 22, "far": 120 },
                "sun": { "color": "#ffaa66", "intensity": 2.2, "elevation": 0.14, "azimuth": 2.4 },
                "hemi": { "sky": "#d6c4ff", "ground": "#8a6a4a", "intensity": 1.6 },
                "ground": "#5a6b3c",
                "grass": { "base": "#5a7040", "tip": "#e8c47a", "density": 0.8, "height": 0.45 },
                "path": "#f2d2bd",
                "house": { "wall": "#dcc6ea", "roof": "#6a4c86", "trim": "#fff3e6", "door": "#ffc87a", "window": "#ffb35c" },
                "trees": "blossom",
                "foliage": ["#ffc4d8", "#ffd9e6", "#f6a6c1", "#ffe3ef"],
                "trunk": "#5b3f3a",
                "flowers": ["#ffe08a", "#ff9e7a", "#ffffff", "#ffb3d1"],
                "hedge": "#4a6136",
                "lamp": "#ffb35c",
                "night": false,
                "motes": "#ffd6ae",
                "water": "#b39be0",
                "ponds": 0.15,
                "terrain": { "amp": 3.4, "scale": 52 },
                "places": { "house": 0.6, "tower": 0.25, "cave": 0.05, "arch": 0.1 }
            }),
        ),
        manifest(
            "fog-pines",
            "Fog Pines",
            "White houses under tall pines. The fog keeps every secret twenty metres away.",
            0x0f09_9135,
            open(3),
            json!({
                "sky": { "top": "#9fb6bd", "horizon": "#e2ebe8", "bottom": "#d3dfdb" },
                "fog": { "color": "#d0ddda", "near": 6, "far": 68 },
                "sun": { "color": "#f2f6ff", "intensity": 1.3, "elevation": 0.7, "azimuth": 4.0 },
                "hemi": { "sky": "#e8f2f4", "ground": "#2f4a40", "intensity": 1.4 },
                "ground": "#2f5446",
                "grass": { "base": "#244a3c", "tip": "#8bb096", "density": 0.9, "height": 0.4 },
                "path": "#d9dcd4",
                "house": { "wall": "#f5f5f0", "roof": "#3e5460", "trim": "#ffffff", "door": "#9cc6d8", "window": "#fff3d8" },
                "trees": "pine",
                "foliage": ["#244a3c", "#2f5d4a", "#1d3d32", "#36695a"],
                "trunk": "#4a3a30",
                "flowers": ["#eef4ff", "#c9e3d8", "#f6f0d0"],
                "hedge": "#2a4d40",
                "lamp": "#eaf4ff",
                "night": false,
                "motes": "#ffffff",
                "water": "#8db4ae",
                "ponds": 0.4,
                "terrain": { "amp": 7.0, "scale": 70 },
                "places": { "house": 0.45, "tower": 0.2, "cave": 0.3, "arch": 0.05 }
            }),
        ),
        manifest(
            "salt-flat-noon",
            "Salt Flat Noon",
            "Pink salt, white houses, palms and a mirage that never resolves. An island: no doors lead out.",
            0x5a17_f1a7,
            PortalPolicy { mode: PolicyMode::Closed, max_per_room: 0, allow_hosts: vec![], allow_local: false },
            json!({
                "sky": { "top": "#7fcaff", "horizon": "#ffe7ef", "bottom": "#ffe3ea" },
                "fog": { "color": "#ffe2e9", "near": 40, "far": 230 },
                "sun": { "color": "#ffffff", "intensity": 2.6, "elevation": 1.25, "azimuth": 1.0 },
                "hemi": { "sky": "#e6f6ff", "ground": "#f2c9c4", "intensity": 1.3 },
                "ground": "#f4d6d2",
                "grass": { "base": "#e3bfba", "tip": "#fff3ec", "density": 0.18, "height": 0.3 },
                "path": "#ffffff",
                "house": { "wall": "#ffffff", "roof": "#ffb3c3", "trim": "#fff6f8", "door": "#78d6df", "window": "#e8fbff" },
                "trees": "palm",
                "foliage": ["#5fb985", "#85d49c", "#4aa678"],
                "trunk": "#c9a27f",
                "flowers": ["#ff9fb5", "#ffd1dc", "#ffffff"],
                "hedge": "#efb8c2",
                "lamp": "#ffffff",
                "night": false,
                "motes": "#ffffff",
                "water": "#9ff0ee",
                "ponds": 0.5,
                "terrain": { "amp": 0.5, "scale": 90 },
                "places": { "house": 0.45, "tower": 0.1, "cave": 0.05, "arch": 0.4 }
            }),
        ),
        manifest(
            "moonlit-meadow",
            "Moonlit Meadow",
            "A night that stays. Glowing mushrooms, fireflies, and every window lit for you.",
            0x3007_1157,
            PortalPolicy { mode: PolicyMode::Allowlist, max_per_room: 2, allow_hosts: vec![], allow_local: true },
            json!({
                "sky": { "top": "#070b26", "horizon": "#2b3b6e", "bottom": "#141c40" },
                "fog": { "color": "#18214a", "near": 14, "far": 95 },
                "sun": { "color": "#a8bbff", "intensity": 1.5, "elevation": 0.8, "azimuth": 1.2 },
                "hemi": { "sky": "#6a7ac8", "ground": "#1a3040", "intensity": 1.5 },
                "ground": "#10291f",
                "grass": { "base": "#0f2a24", "tip": "#3f8a76", "density": 0.9, "height": 0.5 },
                "path": "#3a4470",
                "house": { "wall": "#c3cbe6", "roof": "#2a3052", "trim": "#e6ecff", "door": "#ffd27a", "window": "#ffcd76" },
                "trees": "mushroom",
                "foliage": ["#6ff0ff", "#c39bff", "#ff9be0", "#9bffcf"],
                "trunk": "#d9e4ff",
                "flowers": ["#7ef0ff", "#c39bff", "#fff3a8"],
                "hedge": "#163a33",
                "lamp": "#ffd27a",
                "night": true,
                "motes": "#d8ff7a",
                "water": "#2b4f96",
                "ponds": 0.3,
                "terrain": { "amp": 4.2, "scale": 46 },
                "places": { "house": 0.45, "tower": 0.1, "cave": 0.3, "arch": 0.15 }
            }),
        ),
    ]
}

fn str_hash(s: &str) -> u64 {
    s.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ b as u64).wrapping_mul(0x100_0000_01b3))
}

/// Build the room nobody has touched yet. The starter worlds are pre-wired
/// with a sparse web of doors so the graph exists before anyone builds.
///
/// Every generator door is one half of a physical pair: if house (x,z) in
/// world A has a door to world B, then house (x,z) in B has the door back,
/// and each world's spawn house is joined to every other spawn house. A
/// world that is closed to outbound doors still receives them, but its
/// side of each pair is sealed: you can walk in, not out.
pub fn default_room(world: &WorldManifest, others: &[WorldManifest], x: i32, z: i32) -> Room {
    let seed = (hash(world.seed, x, z, 1) & 0xffff_ffff) as u32;
    let theme = &THEMES[(hash(world.seed, x, z, 2) % THEMES.len() as u64) as usize];
    let mut rng = Rng::new(seed as u64);
    let entry_tag = theme.tags[0];
    let chambers = vec![Chamber {
        name: "Entry Hall".into(),
        tag: entry_tag.into(),
        seed,
        by: "generator".into(),
        at: 0,
    }];

    let policy = &world.portal_policy;
    let sealed = policy.mode == PolicyMode::Closed || (policy.mode == PolicyMode::Allowlist && !policy.allow_local);
    let my_spawn = parse_addr(&world.spawn);
    let mut portals = Vec::new();
    for o in others.iter().filter(|o| world.hub && o.hub) {
        let their_spawn = parse_addr(&o.spawn);
        let (target, slot) = if my_spawn == Some((x, z)) {
            let Some((tx, tz)) = their_spawn else { continue };
            ((tx, tz), 0)
        } else {
            if their_spawn == Some((x, z)) {
                continue;
            }
            let (a, b) = if world.id < o.id { (&world.id, &o.id) } else { (&o.id, &world.id) };
            let pair = splitmix(str_hash(a) ^ splitmix(str_hash(b)));
            let h = hash((pair & 0xffff_ffff) as u32, x, z, 0xB0B);
            if h % 9 != 0 {
                continue;
            }
            ((x, z), ((h >> 8) % 3) as u32)
        };
        portals.push(Portal {
            id: format!("pair-{}", o.id),
            slot,
            target: format!("/w/{}/{},{}", o.id, target.0, target.1),
            label: o.name.clone(),
            by: "generator".into(),
            at: 0,
            sealed,
        });
    }

    let mut weights = BTreeMap::new();
    for t in theme.tags {
        weights.insert(t.to_string(), 1.0 + rng.f32() * 0.2);
    }

    Room {
        x,
        z,
        seed,
        theme: theme.id.into(),
        chambers,
        features: Vec::new(),
        portals,
        log: Vec::new(),
        claim: None,
        attention: 0.0,
        weights,
        investors: HashMap::new(),
        building: None,
        last_visit: HashMap::new(),
        things: Vec::new(),
        looks: BTreeMap::new(),
        touches: BTreeMap::new(),
        replunged: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn others(skip: &str) -> Vec<WorldManifest> {
        builtin_worlds().into_iter().filter(|w| w.id != skip).collect()
    }

    #[test]
    fn rooms_are_deterministic() {
        let w = &builtin_worlds()[0];
        let a = default_room(w, &others(&w.id), 7, -3);
        let b = default_room(w, &others(&w.id), 7, -3);
        assert_eq!((a.seed, a.theme.clone(), a.portals.len()), (b.seed, b.theme, b.portals.len()));
    }

    #[test]
    fn spawn_is_a_hub_and_islands_are_sealed() {
        for w in builtin_worlds() {
            let r = default_room(&w, &others(&w.id), 0, 0);
            assert_eq!(r.portals.len(), 4, "{} hub", w.id);
            let island = w.portal_policy.mode == PolicyMode::Closed;
            assert!(r.portals.iter().all(|p| p.sealed == island));
        }
    }

    #[test]
    fn every_door_has_a_door_back() {
        let worlds = builtin_worlds();
        let mut doors = 0;
        for w in &worlds {
            for x in -6..6 {
                for z in -6..6 {
                    for p in default_room(w, &others(&w.id), x, z).portals {
                        doors += 1;
                        let rest = p.target.strip_prefix("/w/").unwrap();
                        let (tw, addr) = rest.split_once('/').unwrap();
                        let (tx, tz) = parse_addr(addr).unwrap();
                        let t = worlds.iter().find(|m| m.id == tw).unwrap();
                        let back = default_room(t, &others(tw), tx, tz);
                        let want = format!("/w/{}/{},{}", w.id, x, z);
                        assert!(back.portals.iter().any(|b| b.target == want), "{} {x},{z} -> {}", w.id, p.target);
                    }
                }
            }
        }
        assert!(doors > 40);
    }

    #[test]
    fn requests_map_to_tags() {
        assert_eq!(tags_in_text("more WATER and some ferns"), vec!["water", "plants"]);
        assert!(tags_in_text("hello").is_empty());
    }
}

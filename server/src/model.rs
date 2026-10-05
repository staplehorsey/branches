//! Core data model. Everything here is serialisable so the whole universe a
//! host owns can be snapshotted to disk and served over the federation API.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

pub type Tag = String;

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn new_id() -> String {
    uuid::Uuid::new_v4().simple().to_string()[..12].to_string()
}

/// How a world renders. `kind` names a client-side generator
/// (e.g. `liminal-houses@1`); `params` is opaque to the host.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Generator {
    pub kind: String,
    pub params: Value,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyMode {
    /// Any owner may add portals to any host.
    Open,
    /// Portals may only target this host (if `allow_local`) or `allow_hosts`.
    Allowlist,
    /// No new outbound portals. The world is a sink / island.
    Closed,
}

/// World-level limits on growing the graph outward.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PortalPolicy {
    pub mode: PolicyMode,
    pub max_per_room: u32,
    #[serde(default)]
    pub allow_hosts: Vec<String>,
    #[serde(default = "yes")]
    pub allow_local: bool,
}

fn yes() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldManifest {
    pub id: String,
    pub name: String,
    pub tagline: String,
    /// 32-bit so JavaScript clients can use it losslessly.
    pub seed: u32,
    pub generator: Generator,
    pub portal_policy: PortalPolicy,
    pub architect: String,
    /// Address a fresh arrival spawns at, `x,z`.
    pub spawn: String,
    /// Hub worlds are joined spawn-to-spawn and by doors in untouched
    /// houses. Grown worlds are not: they hang off the door that made them.
    #[serde(default = "yes")]
    pub hub: bool,
    /// The world this one grew out of, and the shape of the paths that grew it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<Value>,
}

/// A door in a room that leads somewhere else in the graph.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Portal {
    pub id: String,
    /// Chamber index the door sits in. It only appears once that chamber exists.
    pub slot: u32,
    /// `/w/<world>/<x>,<z>` (same host) or `https://host/w/<world>/<x>,<z>`.
    pub target: String,
    pub label: String,
    pub by: String,
    pub at: u64,
    /// One-way: the door is visible from this side but cannot be opened.
    #[serde(default)]
    pub sealed: bool,
}

/// Something the architect (or an agent) built inside a chamber.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Feature {
    pub id: String,
    pub tag: Tag,
    pub chamber: u32,
    pub seed: u32,
    pub by: String,
    pub at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Chamber {
    pub name: String,
    pub tag: Tag,
    pub seed: u32,
    pub by: String,
    pub at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: String,
    /// visit | note | request | praise | growth | claim | portal
    pub kind: String,
    pub who: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player: Option<String>,
    pub text: String,
    pub at: u64,
}

/// Owner-chosen rule that can only *tighten* the world's portal policy.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct OutboundRule {
    #[serde(default)]
    pub max: Option<u32>,
    #[serde(default)]
    pub allow_hosts: Option<Vec<String>>,
    #[serde(default)]
    pub closed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claim {
    pub owner: String,
    pub owner_name: String,
    pub title: String,
    pub since: u64,
    #[serde(default)]
    pub outbound: OutboundRule,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Build {
    pub started: u64,
    pub ready_at: u64,
}

/// One house/room at a grid address. Only rooms that something has happened
/// to are stored; untouched ones are synthesised deterministically.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Room {
    pub x: i32,
    pub z: i32,
    pub seed: u32,
    pub theme: String,
    pub chambers: Vec<Chamber>,
    pub features: Vec<Feature>,
    pub portals: Vec<Portal>,
    pub log: Vec<LogEntry>,
    pub claim: Option<Claim>,
    /// Seconds of attention not yet spent on growth.
    pub attention: f64,
    /// What this room has learned visitors want more of.
    pub weights: BTreeMap<Tag, f32>,
    /// Player id -> seconds invested here.
    pub investors: HashMap<String, f64>,
    pub building: Option<Build>,
    #[serde(default)]
    pub last_visit: HashMap<String, u64>,
    /// Characters, notes and games living in this room.
    #[serde(default)]
    pub things: Vec<Thing>,
    /// Seconds spent looking at each feature or thing, by id.
    #[serde(default)]
    pub looks: BTreeMap<String, f32>,
    /// Times each feature or thing was used (talked to, read, played).
    #[serde(default)]
    pub touches: BTreeMap<String, u32>,
    /// Brought forward from an older version when someone built on it there.
    #[serde(default)]
    pub replunged: bool,
}

/// Something with a story in a room: a character, a note, a little game.
/// `data` is free-form so architects can invent new kinds; the client
/// renders the kinds it knows and shows the rest as a curious object.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Thing {
    pub id: String,
    pub kind: String,
    pub chamber: u32,
    pub seed: u32,
    #[serde(default)]
    pub data: Value,
    pub by: String,
    pub at: u64,
}

/// How much a cell has been explored.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Heat {
    /// Seconds spent inside the house.
    #[serde(default)]
    pub inside: f32,
    /// Seconds spent outside it, on its lawn and street.
    #[serde(default)]
    pub outside: f32,
    #[serde(default)]
    pub visits: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Version {
    pub tag: String,
    pub at: u64,
    pub summary: String,
}

impl Room {
    pub fn growth(&self) -> u32 {
        self.chambers.len().saturating_sub(1) as u32
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub at: u64,
    pub world: String,
    pub x: i32,
    pub z: i32,
    pub text: String,
    pub read: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    pub id: String,
    pub secret: String,
    pub name: String,
    pub color: String,
    /// Learned global taste profile: what this player lingers in and admires.
    pub prefs: BTreeMap<Tag, f32>,
    pub inbox: Vec<Notification>,
    #[serde(default)]
    pub created: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct World {
    pub manifest: WorldManifest,
    /// Keyed by `x,z`.
    pub rooms: BTreeMap<String, Room>,
    pub log: Vec<LogEntry>,
    /// Exploration per cell, keyed by `x,z`.
    #[serde(default)]
    pub heat: BTreeMap<String, Heat>,
    /// Tagged versions, newest first.
    #[serde(default)]
    pub versions: Vec<Version>,
    /// Recent path signatures of people wandering here.
    #[serde(default)]
    pub paths: Vec<Value>,
    #[serde(default)]
    pub builds_since_tag: u32,
    #[serde(default)]
    pub fresh_cells: u32,
    /// Average seconds per house visit (exponentially weighted).
    #[serde(default)]
    pub pace: f32,
}

impl World {
    pub fn new(manifest: WorldManifest) -> Self {
        World { manifest, rooms: BTreeMap::new(), log: vec![], heat: BTreeMap::new(), versions: vec![], paths: vec![], builds_since_tag: 0, fresh_cells: 0, pace: 0.0 }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Universe {
    pub worlds: BTreeMap<String, World>,
    pub players: HashMap<String, Player>,
}

pub fn addr(x: i32, z: i32) -> String {
    format!("{x},{z}")
}

pub fn parse_addr(s: &str) -> Option<(i32, i32)> {
    let (a, b) = s.split_once(',')?;
    Some((a.trim().parse().ok()?, b.trim().parse().ok()?))
}

/// Push onto a log, keeping it bounded.
pub fn push_log(log: &mut Vec<LogEntry>, entry: LogEntry) {
    log.push(entry);
    if log.len() > 300 {
        let drop = log.len() - 300;
        log.drain(0..drop);
    }
}

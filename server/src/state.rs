//! Shared host state: the universe plus live multiplayer plumbing.

use crate::architect::{self, Architect};
use crate::procgen;
use crate::model::*;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::{broadcast, mpsc};

pub struct Config {
    pub port: u16,
    pub data_dir: PathBuf,
    pub client_dir: PathBuf,
    /// Seconds of attention the first growth in a room costs.
    pub pace: f64,
    pub admin_key: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        let env = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
        let client_dir = env("CLIENT_DIR").map(PathBuf::from).unwrap_or_else(|| {
            ["client", "../client"].iter().map(PathBuf::from).find(|p| p.join("index.html").exists()).unwrap_or_else(|| "client".into())
        });
        Config {
            port: env("PORT").and_then(|p| p.parse().ok()).unwrap_or(8080),
            data_dir: env("DATA_DIR").map(PathBuf::from).unwrap_or_else(|| "data".into()),
            client_dir,
            pace: env("ARCHITECT_PACE").and_then(|p| p.parse().ok()).unwrap_or(40.0),
            admin_key: env("ADMIN_KEY"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Presence {
    pub id: String,
    pub name: String,
    pub color: String,
    pub p: [f32; 3],
    pub ry: f32,
    pub room: Option<String>,
    pub ch: Option<u32>,
    pub last_move: std::time::Instant,
    pub conn: u64,
}

pub type Msg = Arc<str>;

pub struct App {
    pub cfg: Config,
    pub uni: Mutex<Universe>,
    pub hubs: Mutex<HashMap<String, broadcast::Sender<Msg>>>,
    pub presence: Mutex<HashMap<String, HashMap<String, Presence>>>,
    pub direct: Mutex<HashMap<String, Vec<mpsc::UnboundedSender<Msg>>>>,
    pub dirty: AtomicBool,
    pub architect: Box<dyn Architect>,
}

impl App {
    pub fn new(cfg: Config, uni: Universe) -> Arc<Self> {
        Arc::new(App {
            cfg,
            uni: Mutex::new(uni),
            hubs: Mutex::new(HashMap::new()),
            presence: Mutex::new(HashMap::new()),
            direct: Mutex::new(HashMap::new()),
            dirty: AtomicBool::new(false),
            architect: Box::new(architect::Heuristic),
        })
    }

    pub fn hub(&self, world: &str) -> broadcast::Sender<Msg> {
        self.hubs
            .lock()
            .unwrap()
            .entry(world.to_string())
            .or_insert_with(|| broadcast::channel(256).0)
            .clone()
    }

    pub fn broadcast(&self, world: &str, msg: &Value) {
        let _ = self.hub(world).send(Arc::from(msg.to_string()));
    }

    pub fn send_direct(&self, player: &str, msg: &Value) {
        let text: Msg = Arc::from(msg.to_string());
        if let Some(conns) = self.direct.lock().unwrap().get_mut(player) {
            conns.retain(|tx| tx.send(text.clone()).is_ok());
        }
    }

    pub fn touch(&self) {
        self.dirty.store(true, Ordering::Relaxed);
    }

    pub fn threshold(&self, growth: u32) -> f64 {
        self.cfg.pace * (1.0 + 0.6 * growth as f64)
    }
}

impl Universe {
    pub fn seed_builtins(&mut self) {
        for m in procgen::builtin_worlds() {
            match self.worlds.get_mut(&m.id) {
                // Keep rooms and logs, refresh the look and rules from code.
                Some(w) => w.manifest = m,
                None => {
                    self.worlds.insert(m.id.clone(), World { manifest: m, rooms: Default::default(), log: vec![] });
                }
            }
        }
    }

    pub fn others(&self, world: &str) -> Vec<(String, String)> {
        self.worlds
            .values()
            .filter(|w| w.manifest.id != world)
            .map(|w| (w.manifest.id.clone(), w.manifest.name.clone()))
            .collect()
    }

    /// Stored room or the deterministic default.
    pub fn room(&self, world: &str, x: i32, z: i32) -> Option<Room> {
        let w = self.worlds.get(world)?;
        Some(match w.rooms.get(&addr(x, z)) {
            Some(r) => r.clone(),
            None => procgen::default_room(&w.manifest, &self.others(world), x, z),
        })
    }

    /// Mutable room, materialising the default on first write.
    pub fn room_mut(&mut self, world: &str, x: i32, z: i32) -> Option<&mut Room> {
        let others = self.others(world);
        let w = self.worlds.get_mut(world)?;
        let manifest = &w.manifest;
        Some(w.rooms.entry(addr(x, z)).or_insert_with(|| procgen::default_room(manifest, &others, x, z)))
    }

    pub fn auth(&self, pid: &str, secret: &str) -> Option<&Player> {
        self.players.get(pid).filter(|p| p.secret == secret)
    }
}

/// Compact view used when streaming neighbourhoods.
pub fn room_summary(r: &Room) -> Value {
    json!({
        "x": r.x, "z": r.z, "seed": r.seed, "theme": r.theme,
        "growth": r.growth(),
        "claimed": r.claim.as_ref().map(|c| c.title.clone()),
        "portals": r.portals.len(),
        "building": r.building.is_some(),
    })
}

/// Full public view of a room. Investor ids and secrets never leave the host.
pub fn room_view(app: &App, uni: &Universe, world: &str, r: &Room) -> Value {
    let mut investors: Vec<(&String, &f64)> = r.investors.iter().collect();
    investors.sort_by(|a, b| b.1.total_cmp(a.1));
    let top: Vec<Value> = investors
        .iter()
        .take(8)
        .map(|(pid, secs)| {
            let name = uni.players.get(*pid).map(|p| p.name.clone()).unwrap_or_else(|| "someone".into());
            json!({ "name": name, "seconds": secs.round() })
        })
        .collect();
    let leaning = architect::leaning(r, uni, 3);
    json!({
        "world": world,
        "x": r.x, "z": r.z, "seed": r.seed, "theme": r.theme,
        "theme_name": procgen::theme(&r.theme).map(|t| t.name).unwrap_or("Unknown"),
        "chambers": r.chambers,
        "features": r.features,
        "portals": r.portals,
        "log": r.log.iter().rev().take(60).collect::<Vec<_>>(),
        "claim": r.claim,
        "growth": r.growth(),
        "attention": r.attention.round(),
        "next_growth_at": app.threshold(r.growth()).round(),
        "building": r.building,
        "leaning": leaning,
        "investors": top,
    })
}

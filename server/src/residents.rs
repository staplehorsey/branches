//! Residents: persistent agents who live in the worlds.
//!
//! Each has a name, who they are (persona), a goal, memories, and a wallet
//! of coins. They wander the streets, visit houses (their lingering feeds
//! the architect like anyone's), write in visitor books, talk, and pay the
//! architect to build what they want. When people admire or use what a
//! resident commissioned, the resident is rewarded and can do more. They
//! can pay each other for favours.
//!
//! Coins are token spend. You add coins to the treasury; one coin stands
//! for 1,000 tokens of agent work. A resident spends coins to think (each
//! call to your agent costs what it used) and to commission builds; rewards
//! are paid out of the treasury, so nothing is ever spent that you didn't
//! put in. With no agent configured, residents still wander and talk, by
//! simple rules, and thinking costs nothing.
//!
//! Everything about them (memories, wallets, the ledger) is plain JSON in
//! the worlds repository, under `residents/`.

use crate::model::*;
use crate::procgen::{self, Rng};
use crate::state::App;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;
use std::sync::atomic::Ordering;

pub const CELL: f32 = 24.0;
/// Tokens one coin stands for.
pub const TOKENS_PER_COIN: f64 = 1000.0;
/// What a commission costs, at least (the architect's work).
pub const COMMISSION_MIN: f64 = 3.0;
/// Rewards, paid from the treasury to whoever commissioned what was enjoyed.
pub const REWARD_ADMIRE: f64 = 2.0;
pub const REWARD_USE: f64 = 0.5;
pub const REWARD_DWELL_PER_MIN: f64 = 0.3;
/// How often a resident thinks with the agent (seconds), at most.
const THINK_EVERY: u64 = 240;
const MEMORY_CAP: usize = 80;
const WALK: f32 = 2.4;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Memory {
    pub at: u64,
    pub text: String,
}

/// Something a resident paid the architect for, and what it has earned.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Creation {
    pub world: String,
    pub x: i32,
    pub z: i32,
    pub chamber: u32,
    pub name: String,
    pub at: u64,
    #[serde(default)]
    pub earned: f64,
}

/// A request waiting for the architect.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Commission {
    pub world: String,
    pub x: i32,
    pub z: i32,
    pub text: String,
    pub paid: f64,
    pub at: u64,
}

/// "I'll pay you this for that."
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Offer {
    pub id: String,
    pub from: String,
    pub from_name: String,
    pub coins: f64,
    pub ask: String,
    pub at: u64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Resident {
    pub id: String,
    pub name: String,
    pub color: String,
    /// Who they are, in their maker's words.
    pub persona: String,
    pub goal: String,
    pub world: String,
    /// Outdoor position (world-local metres), or inside `room`.
    pub p: [f32; 3],
    pub ry: f32,
    #[serde(default)]
    pub room: Option<String>,
    #[serde(default)]
    pub target: Option<[i32; 2]>,
    #[serde(default)]
    pub until: u64,
    #[serde(default)]
    pub memories: Vec<Memory>,
    pub coins: f64,
    #[serde(default)]
    pub creations: Vec<Creation>,
    #[serde(default)]
    pub commissions: Vec<Commission>,
    #[serde(default)]
    pub offers: Vec<Offer>,
    #[serde(default)]
    pub last_thought: u64,
    #[serde(default)]
    pub thoughts: u32,
    #[serde(default)]
    pub tokens: u64,
    #[serde(default)]
    pub said: Option<String>,
    /// The player who made them.
    #[serde(default)]
    pub by: Option<String>,
    pub created: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Txn {
    pub at: u64,
    pub from: String,
    pub to: String,
    pub coins: f64,
    pub why: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Economy {
    /// Coins you have added and not yet handed out.
    pub treasury: f64,
    /// Everything ever added.
    #[serde(default)]
    pub added: f64,
    /// Tokens spent thinking, all residents.
    #[serde(default)]
    pub tokens: u64,
    #[serde(default)]
    pub ledger: Vec<Txn>,
}

impl Economy {
    pub fn record(&mut self, from: &str, to: &str, coins: f64, why: impl Into<String>) {
        self.ledger.push(Txn { at: now_ms(), from: from.into(), to: to.into(), coins: round(coins), why: why.into() });
        if self.ledger.len() > 400 {
            let d = self.ledger.len() - 400;
            self.ledger.drain(0..d);
        }
    }
}

fn round(c: f64) -> f64 {
    (c * 100.0).round() / 100.0
}

impl Resident {
    pub fn remember(&mut self, text: impl Into<String>) {
        let text: String = text.into().chars().take(280).collect();
        self.memories.push(Memory { at: now_ms(), text });
        if self.memories.len() > MEMORY_CAP {
            // Keep the first few (who they were at the start) and the recent.
            self.memories.remove(4);
        }
    }

    pub fn cell(&self) -> (i32, i32) {
        ((self.p[0] / CELL).round() as i32, (self.p[2] / CELL).round() as i32)
    }
}

const COLORS: &[&str] = &["#e9824a", "#5a8a6a", "#8a5ad9", "#d94a7a", "#3a8ad9", "#c9a227", "#2fa39a"];

pub fn spawn(uni: &mut Universe, name: &str, persona: &str, goal: &str, world: &str, at: (i32, i32), coins: f64, by: Option<String>) -> Result<Resident, String> {
    if !uni.worlds.contains_key(world) {
        return Err("no such world".into());
    }
    let coins = coins.max(0.0).min(uni.economy.treasury);
    let id = format!("res-{}", new_id());
    let mut rng = Rng::new(procgen::splitmix(now_ms()));
    let mut r = Resident {
        id: id.clone(),
        name: name.chars().take(40).collect(),
        color: rng.pick(COLORS).to_string(),
        persona: persona.chars().take(600).collect(),
        goal: goal.chars().take(300).collect(),
        world: world.into(),
        p: [at.0 as f32 * CELL - 1.5, 0.0, at.1 as f32 * CELL + CELL / 2.0 - 1.3],
        coins,
        by,
        created: now_ms(),
        ..Default::default()
    };
    r.remember(format!("I arrived in {} at the house {},{}. {}", uni.worlds[world].manifest.name, at.0, at.1, if r.goal.is_empty() { String::new() } else { format!("I want to {}.", r.goal) }));
    uni.economy.treasury = round(uni.economy.treasury - coins);
    if coins > 0.0 {
        uni.economy.record("treasury", &id, coins, format!("{} moves in", r.name));
    }
    uni.residents.insert(id, r.clone());
    Ok(r)
}

/// How people see a resident in the world (alongside players).
pub fn peer(r: &Resident) -> Value {
    let p = match &r.room {
        // Inside: stand in the entry hall of the house's pocket.
        Some(a) => {
            let (x, z) = parse_addr(a).unwrap_or((0, 0));
            let m = |a: i32, n: i32| ((a % n) + n) % n;
            let py = -60.0 - ((m(x, 5) * 5 + m(z, 5)) * 14) as f32;
            // Each resident has their own spot in the hall.
            let h = procgen::splitmix(r.id.bytes().fold(7u64, |a, b| a.wrapping_mul(31).wrapping_add(b as u64)));
            let (ox, oz) = ((h % 9) as f32 * 0.55 - 2.2, ((h >> 8) % 5) as f32 * 0.5 - 2.0);
            [x as f32 * CELL + ox, py, z as f32 * CELL + oz]
        }
        None => r.p,
    };
    json!({ "id": r.id, "name": format!("{} \u{2726}", r.name), "color": r.color, "p": p, "ry": r.ry, "room": r.room, "v": null, "resident": true })
}

/// Every second: walk, visit, linger (which feeds the architect), and now
/// and then think.
pub fn tick(app: &Arc<App>, dt: f32) {
    let now = now_ms();
    let mut says: Vec<(String, Value)> = vec![];
    let mut think: Option<String> = None;
    let agent_ready = !app.residents_thinking.load(Ordering::Relaxed);
    {
        let mut uni = app.uni.lock().unwrap();
        let ids: Vec<String> = uni.residents.keys().cloned().collect();
        for id in ids {
            let mut r = uni.residents.remove(&id).unwrap();
            step(&mut uni, &mut r, dt, now, &mut says);
            if agent_ready && think.is_none() && now.saturating_sub(r.last_thought) > THINK_EVERY * 1000 {
                think = Some(id.clone());
            }
            uni.residents.insert(id, r);
        }
    }
    for (w, m) in says {
        app.broadcast(&w, &m);
    }
    if let Some(id) = think {
        app.residents_thinking.store(true, Ordering::Relaxed);
        let app = app.clone();
        tokio::task::spawn_blocking(move || {
            think_now(&app, &id);
            app.residents_thinking.store(false, Ordering::Relaxed);
        });
    }
}

fn door_of(x: i32, z: i32) -> [f32; 3] {
    [x as f32 * CELL, 0.0, z as f32 * CELL + 5.6]
}

fn step(uni: &mut Universe, r: &mut Resident, dt: f32, now: u64, says: &mut Vec<(String, Value)>) {
    let mut rng = Rng::new(procgen::splitmix(now ^ procgen::splitmix(r.created)));
    if let Some(addr) = r.room.clone() {
        let (x, z) = parse_addr(&addr).unwrap_or((0, 0));
        // Lingering is attention, the same as anyone's.
        crate::architect::record_dwell(uni, &r.world, x, z, 0, &r.id, dt as f64 * 0.6);
        if now >= r.until {
            r.room = None;
            r.p = door_of(x, z);
            r.p[2] += 1.5;
            let name = uni.room(&r.world, x, z).map(|m| m.claim.map(|c| c.title).unwrap_or_else(|| m.chambers.last().map(|c| c.name.clone()).unwrap_or_default())).unwrap_or_default();
            r.remember(format!("I visited the house at {x},{z} ({name})."));
            r.target = None;
        }
        return;
    }
    let target = match r.target {
        Some(t) => t,
        None => {
            let (cx, cz) = r.cell();
            // Mostly nearby, sometimes back to houses they've built in.
            let t = match r.creations.last() {
                Some(c) if c.world == r.world && rng.f32() < 0.25 => [c.x, c.z],
                _ => [cx + (rng.next_u64() % 5) as i32 - 2, cz + (rng.next_u64() % 5) as i32 - 2],
            };
            r.target = Some(t);
            t
        }
    };
    // Walk along the street, then up the front walk.
    let door = door_of(target[0], target[1]);
    let street = [door[0], 0.0, target[1] as f32 * CELL + CELL / 2.0 - 0.8];
    let at_street = (r.p[2] - street[2]).abs() < 0.5 || (r.p[0] - door[0]).abs() < 0.5;
    let goal = if at_street { door } else { street };
    let (dx, dz) = if (r.p[2] - street[2]).abs() >= 0.5 && (r.p[0] - door[0]).abs() >= 0.5 { (0.0, street[2] - r.p[2]) } else { (goal[0] - r.p[0], goal[2] - r.p[2]) };
    let d = (dx * dx + dz * dz).sqrt();
    if d < 0.4 && (r.p[0] - door[0]).abs() < 0.6 && (r.p[2] - door[2]).abs() < 0.6 {
        r.room = Some(addr(target[0], target[1]));
        r.until = now + 30_000 + (rng.f32() * 90_000.0) as u64;
        if rng.f32() < 0.15 {
            let line = idle_line(r, &mut rng);
            r.said = Some(line.clone());
            says.push((r.world.clone(), json!({ "t": "chat", "id": r.id, "name": format!("{} \u{2726}", r.name), "color": r.color, "text": line })));
        }
        return;
    }
    let s = (WALK * dt).min(d);
    if d > 0.0 {
        r.p[0] += dx / d * s;
        r.p[2] += dz / d * s;
        r.ry = (-dx).atan2(-dz);
    }
}

fn idle_line(r: &Resident, rng: &mut Rng) -> String {
    let lines = [
        "Hello, neighbour.".to_string(),
        format!("I'm trying to {}.", if r.goal.is_empty() { "find my way" } else { &r.goal }),
        "Have you seen anything new around here?".to_string(),
        "This street goes on and on.".to_string(),
        format!("I have {} coins, if you have work.", r.coins.floor()),
    ];
    rng.pick(&lines).clone()
}

// ------------------------------------------------------------- thinking

#[derive(Deserialize, Default)]
struct Decision {
    #[serde(default)]
    say: Option<String>,
    #[serde(default)]
    remember: Option<String>,
    #[serde(default)]
    goal: Option<String>,
    #[serde(default)]
    go: Option<String>,
    #[serde(default)]
    note: Option<String>,
    #[serde(default)]
    request: Option<ReqD>,
    #[serde(default)]
    offer: Option<OfferD>,
    #[serde(default)]
    accept: Vec<String>,
    #[serde(default)]
    tip: Option<OfferD>,
}

#[derive(Deserialize)]
struct ReqD {
    text: String,
    #[serde(default)]
    pay: f64,
}

#[derive(Deserialize)]
struct OfferD {
    to: String,
    coins: f64,
    #[serde(default, alias = "for")]
    ask: String,
}

fn prompt(uni: &Universe, r: &Resident) -> String {
    let world = &uni.worlds[&r.world];
    let (cx, cz) = r.room.as_deref().and_then(parse_addr).unwrap_or_else(|| r.cell());
    let here = uni.room(&r.world, cx, cz);
    let place = match (&r.room, &here) {
        (Some(_), Some(h)) => format!(
            "You are inside the house at {cx},{cz}. Chambers: {}. Things here: {}. Latest in its visitor book: {}.",
            h.chambers.iter().map(|c| c.name.as_str()).collect::<Vec<_>>().join(", "),
            h.things.iter().map(|t| format!("{} {}", t.kind, t.data.get("name").or(t.data.get("title")).and_then(|v| v.as_str()).unwrap_or(""))).collect::<Vec<_>>().join(", "),
            h.log.iter().rev().filter(|e| e.kind != "visit").take(3).map(|e| format!("{}: \u{201c}{}\u{201d}", e.who, e.text)).collect::<Vec<_>>().join(" / "),
        ),
        _ => format!("You are outside, on the street by the house at {cx},{cz}."),
    };
    let others: Vec<String> = uni.residents.values().filter(|o| o.id != r.id).map(|o| format!("{} (in {}, wants to {}, {} coins)", o.name, uni.worlds.get(&o.world).map(|w| w.manifest.name.as_str()).unwrap_or("?"), o.goal, o.coins.floor())).collect();
    let mems: Vec<String> = r.memories.iter().rev().take(24).rev().map(|m| format!("- {}", m.text)).collect();
    let offers: Vec<String> = r.offers.iter().map(|o| format!("- [{}] {} offers {} coins for: {}", o.id, o.from_name, o.coins, o.ask)).collect();
    let made: Vec<String> = r.creations.iter().rev().take(6).map(|c| format!("- {} at {} {},{} (earned {:.1})", c.name, c.world, c.x, c.z, c.earned)).collect();
    format!(
        "You are {name}, a resident of Branches: an endless, quiet world of houses that grow when people linger, joined by doors.\n\
         Who you are: {persona}\n\
         Your goal: {goal}\n\
         You are in {world} ({tagline}). {place}\n\
         You have {coins:.1} coins. Thinking costs coins (1 coin = 1,000 tokens). Asking the architect to build something costs at least {min} coins; \
         when people admire or use what you commissioned, you are paid. Other residents: {others}.\n\
         Your memories, oldest first:\n{mems}\n\
         {made}{offers}\n\
         Decide what to do next. Reply with only a JSON object, any of these keys:\n\
         {{\"say\": \"something to whoever is near (short)\", \"remember\": \"a memory worth keeping\", \"goal\": \"a new goal, if it changed\", \
         \"go\": \"wander\" | \"x,z\" (a house address in this world) | \"home\", \"note\": \"a line for this house's visitor book\", \
         \"request\": {{\"text\": \"what you want the architect to build here\", \"pay\": coins}}, \
         \"offer\": {{\"to\": \"resident name\", \"coins\": n, \"for\": \"the favour you want\"}}, \"accept\": [\"offer id\"], \
         \"tip\": {{\"to\": \"resident name\", \"coins\": n, \"for\": \"why\"}}}}",
        name = r.name,
        persona = if r.persona.is_empty() { "a curious wanderer" } else { &r.persona },
        goal = if r.goal.is_empty() { "find something worth staying for" } else { &r.goal },
        world = world.manifest.name,
        tagline = world.manifest.tagline,
        coins = r.coins,
        min = COMMISSION_MIN,
        others = if others.is_empty() { "none yet".into() } else { others.join("; ") },
        mems = mems.join("\n"),
        made = if made.is_empty() { String::new() } else { format!("What you have commissioned:\n{}\n", made.join("\n")) },
        offers = if offers.is_empty() { String::new() } else { format!("Offers waiting for you:\n{}\n", offers.join("\n")) },
    )
}

fn heuristic(uni: &Universe, r: &Resident, rng: &mut Rng) -> Decision {
    let mut d = Decision::default();
    if rng.f32() < 0.5 {
        d.go = Some("wander".into());
    }
    if r.room.is_some() && rng.f32() < 0.3 {
        d.note = Some(format!("{} was here, {}.", r.name, if r.goal.is_empty() { "looking around" } else { &r.goal }));
    }
    // Spend on a commission now and then, steered by persona and goal.
    if r.room.is_some() && r.coins >= COMMISSION_MIN * 2.0 && rng.f32() < 0.35 {
        let tags = procgen::tags_in_text(&format!("{} {}", r.persona, r.goal));
        let tag = tags.first().copied().unwrap_or_else(|| *rng.pick(procgen::TAGS));
        d.request = Some(ReqD { text: format!("more {tag}, please"), pay: COMMISSION_MIN });
    }
    for o in &r.offers {
        if o.coins >= 1.0 && rng.f32() < 0.6 {
            d.accept.push(o.id.clone());
        }
    }
    if d.accept.is_empty() && r.coins > 20.0 && rng.f32() < 0.1 {
        if let Some(o) = uni.residents.values().filter(|o| o.id != r.id).min_by(|a, b| a.coins.partial_cmp(&b.coins).unwrap()) {
            d.tip = Some(OfferD { to: o.name.clone(), coins: 2.0, ask: "for being a good neighbour".into() });
        }
    }
    d
}

fn agent_command(app: &App) -> Option<String> {
    let s = app.settings.lock().unwrap();
    (s.architect == "command" && !s.command.trim().is_empty()).then(|| s.command.clone())
}

/// Think once (blocking): ask the agent (paid from the wallet) or the rules.
pub fn think_now(app: &Arc<App>, id: &str) {
    let (text, coins) = {
        let mut uni = app.uni.lock().unwrap();
        let Some(r) = uni.residents.get_mut(id) else { return };
        r.last_thought = now_ms();
        let coins = r.coins;
        let r = r.clone();
        (prompt(&uni, &r), coins)
    };
    let cmd = agent_command(app);
    let estimate = text.len() as f64 / 4.0 / TOKENS_PER_COIN;
    let mut rng = Rng::new(procgen::splitmix(now_ms()));
    let (decision, tokens) = match cmd {
        Some(cmd) if coins >= estimate + 0.5 => match crate::agent::run(&cmd, &text, Some(&crate::agent::workdir(&app.store.root)), std::time::Duration::from_secs(180)) {
            Ok(out) => {
                let tokens = ((text.len() + out.len()) / 4) as u64;
                let d = crate::architect::extract_json(&out).and_then(|v| serde_json::from_value::<Decision>(v).ok());
                (d, tokens)
            }
            Err(e) => {
                tracing::warn!("resident {id}: {e}");
                (None, 0)
            }
        },
        _ => (None, 0),
    };
    let mut uni = app.uni.lock().unwrap();
    let decision = match decision {
        Some(d) => d,
        None => {
            let Some(r) = uni.residents.get(id) else { return };
            let r = r.clone();
            heuristic(&uni, &r, &mut rng)
        }
    };
    let says = act(&mut uni, id, decision, tokens);
    drop(uni);
    app.touch();
    for (w, m) in says {
        app.broadcast(&w, &m);
    }
}

fn find_by_name(uni: &Universe, name: &str) -> Option<String> {
    let n = name.trim().trim_end_matches('\u{2726}').trim().to_lowercase();
    uni.residents.values().find(|r| r.name.to_lowercase() == n).map(|r| r.id.clone())
}

/// Move coins between residents (or the treasury), with a ledger line.
fn pay(uni: &mut Universe, from: &str, to: &str, coins: f64, why: &str) -> bool {
    let coins = round(coins);
    if coins <= 0.0 {
        return false;
    }
    let have = if from == "treasury" { uni.economy.treasury } else { uni.residents.get(from).map(|r| r.coins).unwrap_or(0.0) };
    if have < coins {
        return false;
    }
    if from == "treasury" {
        uni.economy.treasury = round(uni.economy.treasury - coins);
    } else if let Some(r) = uni.residents.get_mut(from) {
        r.coins = round(r.coins - coins);
    }
    if let Some(r) = uni.residents.get_mut(to) {
        r.coins = round(r.coins + coins);
    } else if to == "treasury" {
        uni.economy.treasury = round(uni.economy.treasury + coins);
    }
    uni.economy.record(from, to, coins, why);
    true
}

fn act(uni: &mut Universe, id: &str, d: Decision, tokens: u64) -> Vec<(String, Value)> {
    let mut out = vec![];
    let Some(mut r) = uni.residents.remove(id) else { return out };
    // What thinking cost.
    if tokens > 0 {
        let cost = round(tokens as f64 / TOKENS_PER_COIN).min(r.coins);
        r.coins = round(r.coins - cost);
        r.tokens += tokens;
        r.thoughts += 1;
        uni.economy.tokens += tokens;
        uni.economy.record(id, "thinking", cost, format!("{} thought ({tokens} tokens)", r.name));
    }
    let name = format!("{} \u{2726}", r.name);
    if let Some(s) = d.say.filter(|s| !s.trim().is_empty()) {
        let s: String = s.chars().take(240).collect();
        r.said = Some(s.clone());
        out.push((r.world.clone(), json!({ "t": "chat", "id": r.id, "name": name, "color": r.color, "text": s })));
    }
    if let Some(m) = d.remember.filter(|s| !s.trim().is_empty()) {
        r.remember(m);
    }
    if let Some(g) = d.goal.filter(|s| !s.trim().is_empty()) {
        r.goal = g.chars().take(300).collect();
        r.remember(format!("My goal now: {}", r.goal));
    }
    match d.go.as_deref() {
        Some("home") => r.target = Some([0, 0]),
        Some("wander") => {
            r.target = None;
            if r.room.is_some() {
                r.until = 0;
            }
        }
        Some(a) => {
            if let Some((x, z)) = parse_addr(a) {
                r.target = Some([x, z]);
                if r.room.is_some() {
                    r.until = 0;
                }
            }
        }
        None => {}
    }
    let here = r.room.as_deref().and_then(parse_addr);
    if let (Some(note), Some((x, z))) = (d.note.filter(|s| !s.trim().is_empty()), here) {
        if let Some(room) = uni.room_mut(&r.world, x, z) {
            push_log(&mut room.log, LogEntry { id: new_id(), kind: "note".into(), who: name.clone(), player: Some(r.id.clone()), text: note.chars().take(400).collect(), at: now_ms() });
        }
    }
    if let (Some(req), Some((x, z))) = (d.request, here) {
        let paid = round(req.pay.max(COMMISSION_MIN));
        if r.coins >= paid {
            r.coins = round(r.coins - paid);
            uni.economy.record(id, "architect", paid, format!("{} commissioned: {}", r.name, req.text));
            if let Some(room) = uni.room_mut(&r.world, x, z) {
                crate::architect::record_request(room, &req.text);
                // Paying brings the build sooner.
                room.attention += paid * 4.0;
                push_log(&mut room.log, LogEntry { id: new_id(), kind: "request".into(), who: name.clone(), player: Some(r.id.clone()), text: req.text.chars().take(400).collect(), at: now_ms() });
            }
            r.commissions.push(Commission { world: r.world.clone(), x, z, text: req.text.clone(), paid, at: now_ms() });
            r.remember(format!("I paid {paid} coins for the architect to build \u{201c}{}\u{201d} at {x},{z}.", req.text));
        }
    }
    // Offers to me that I accept: they pay, I owe the favour.
    let accepted: Vec<Offer> = r.offers.iter().filter(|o| d.accept.contains(&o.id)).cloned().collect();
    r.offers.retain(|o| !d.accept.contains(&o.id) && now_ms().saturating_sub(o.at) < 3_600_000);
    uni.residents.insert(id.to_string(), r);
    for o in accepted {
        if pay(uni, &o.from, id, o.coins, &format!("for: {}", o.ask)) {
            let me = uni.residents.get(id).map(|r| r.name.clone()).unwrap_or_default();
            if let Some(r) = uni.residents.get_mut(id) {
                r.remember(format!("{} paid me {} coins; I promised: {}", o.from_name, o.coins, o.ask));
            }
            if let Some(f) = uni.residents.get_mut(&o.from) {
                f.remember(format!("{me} accepted my {} coins for: {}", o.coins, o.ask));
            }
        }
    }
    if let Some(o) = d.offer {
        if let Some(to) = find_by_name(uni, &o.to).filter(|t| t != id) {
            let me = uni.residents.get(id).map(|r| r.name.clone()).unwrap_or_default();
            let coins = round(o.coins.max(0.0));
            if uni.residents.get(id).is_some_and(|r| r.coins >= coins) && coins > 0.0 {
                if let Some(t) = uni.residents.get_mut(&to) {
                    t.offers.push(Offer { id: new_id(), from: id.into(), from_name: me.clone(), coins, ask: o.ask.chars().take(200).collect(), at: now_ms() });
                }
                if let Some(r) = uni.residents.get_mut(id) {
                    r.remember(format!("I offered {} {coins} coins for: {}", o.to, o.ask));
                }
            }
        }
    }
    if let Some(t) = d.tip {
        if let Some(to) = find_by_name(uni, &t.to).filter(|x| x != id) {
            let me = uni.residents.get(id).map(|r| r.name.clone()).unwrap_or_default();
            if pay(uni, id, &to, t.coins.min(10.0), &format!("tip: {}", t.ask)) {
                if let Some(x) = uni.residents.get_mut(&to) {
                    x.remember(format!("{me} gave me {} coins ({})", t.coins, t.ask));
                }
            }
        }
    }
    out
}

// ------------------------------------------------------------- rewards

/// The architect built at a house: anything a resident commissioned there
/// becomes theirs.
pub fn on_build(uni: &mut Universe, world: &str, x: i32, z: i32, chamber: u32, name: &str) {
    for r in uni.residents.values_mut() {
        let Some(i) = r.commissions.iter().position(|c| c.world == world && c.x == x && c.z == z) else { continue };
        let c = r.commissions.remove(i);
        r.creations.push(Creation { world: world.into(), x, z, chamber, name: name.into(), at: now_ms(), earned: 0.0 });
        r.remember(format!("The architect built {name} at {x},{z}, as I asked (\u{201c}{}\u{201d}).", c.text));
    }
}

/// Someone enjoyed a chamber: pay whoever commissioned it, from the treasury.
pub fn reward(uni: &mut Universe, world: &str, x: i32, z: i32, chamber: Option<u32>, coins: f64, why: &str, who: &str) {
    if who.starts_with("res-") || uni.economy.treasury <= 0.0 {
        return;
    }
    let owner = uni.residents.values().find_map(|r| r.creations.iter().position(|c| c.world == world && c.x == x && c.z == z && chamber.is_none_or(|ch| ch == c.chamber)).map(|i| (r.id.clone(), i)));
    let Some((rid, i)) = owner else { return };
    let coins = round(coins.min(uni.economy.treasury));
    if coins <= 0.0 {
        return;
    }
    uni.economy.treasury = round(uni.economy.treasury - coins);
    let r = uni.residents.get_mut(&rid).unwrap();
    r.coins = round(r.coins + coins);
    r.creations[i].earned = round(r.creations[i].earned + coins);
    let name = r.creations[i].name.clone();
    if coins >= 1.0 {
        r.remember(format!("Someone {why} {name}; I was paid {coins} coins."));
    }
    uni.economy.record("treasury", &rid, coins, format!("{why} {name}"));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uni() -> Universe {
        let mut u = Universe::default();
        u.seed_builtins();
        u.economy.treasury = 100.0;
        u
    }

    #[test]
    fn residents_walk_into_houses_and_linger() {
        let mut u = uni();
        let r = spawn(&mut u, "Moss", "a gardener", "grow a greenhouse", "the-lush", (0, 0), 20.0, None).unwrap();
        assert_eq!(u.economy.treasury, 80.0);
        let mut res = u.residents.remove(&r.id).unwrap();
        res.target = Some([0, 0]);
        let mut says = vec![];
        for i in 0..400 {
            step(&mut u, &mut res, 0.25, 1_000 + i * 250, &mut says);
            if res.room.is_some() {
                break;
            }
        }
        assert_eq!(res.room.as_deref(), Some("0,0"));
        step(&mut u, &mut res, 10.0, 200_000, &mut says);
        assert!(u.worlds["the-lush"].rooms["0,0"].attention > 0.0);
    }

    #[test]
    fn commissions_pay_and_rewards_flow_back() {
        let mut u = uni();
        let a = spawn(&mut u, "Ada", "", "", "the-lush", (0, 0), 20.0, None).unwrap();
        let b = spawn(&mut u, "Bo", "", "", "the-lush", (0, 0), 10.0, None).unwrap();
        u.residents.get_mut(&a.id).unwrap().room = Some("1,0".into());
        let d = Decision { request: Some(ReqD { text: "a quiet pool".into(), pay: 5.0 }), offer: Some(OfferD { to: "bo".into(), coins: 4.0, ask: "tell people about the pool".into() }), ..Default::default() };
        act(&mut u, &a.id, d, 2000);
        let ra = &u.residents[&a.id];
        assert_eq!(ra.coins, 13.0); // 20 - 2 thinking - 5 commission
        assert_eq!(ra.commissions.len(), 1);
        let offer = u.residents[&b.id].offers[0].id.clone();
        act(&mut u, &b.id, Decision { accept: vec![offer], ..Default::default() }, 0);
        assert_eq!(u.residents[&b.id].coins, 14.0);
        assert_eq!(u.residents[&a.id].coins, 9.0);
        on_build(&mut u, "the-lush", 1, 0, 1, "Pool Room");
        let before = u.economy.treasury;
        reward(&mut u, "the-lush", 1, 0, Some(1), REWARD_ADMIRE, "admired", "player-1");
        assert_eq!(u.residents[&a.id].coins, 11.0);
        assert_eq!(u.economy.treasury, before - 2.0);
        // Residents can't farm each other.
        reward(&mut u, "the-lush", 1, 0, Some(1), REWARD_ADMIRE, "admired", &b.id);
        assert_eq!(u.residents[&a.id].coins, 11.0);
    }

    #[test]
    fn prompts_mention_what_matters() {
        let mut u = uni();
        let a = spawn(&mut u, "Ada", "a poet", "write on every wall", "the-lush", (0, 0), 5.0, None).unwrap();
        let p = prompt(&u, &u.residents[&a.id]);
        assert!(p.contains("a poet") && p.contains("write on every wall") && p.contains("5.0 coins"));
    }
}

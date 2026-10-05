//! HTTP API. This is the federation surface: any client (or agent) that
//! speaks it can walk, grow and link worlds on this host.

use crate::architect;
use crate::procgen;
use crate::model::*;
use crate::state::{App, room_summary, room_view};
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{Json, Router, routing::get, routing::post};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::Arc;

type AppState = State<Arc<App>>;

pub struct ApiError(StatusCode, String);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({ "error": self.1 }))).into_response()
    }
}

fn err(code: StatusCode, msg: impl Into<String>) -> ApiError {
    ApiError(code, msg.into())
}

type Res = Result<Json<Value>, ApiError>;

pub fn router() -> Router<Arc<App>> {
    Router::new()
        .route("/.well-known/branches.json", get(host_manifest))
        .route("/api/themes", get(themes))
        .route("/api/graph", get(graph))
        .route("/api/worlds", get(worlds))
        .route("/api/worlds/{w}", get(world))
        .route("/api/worlds/{w}/chunk", get(chunk))
        .route("/api/worlds/{w}/map", get(map))
        .route("/api/app/settings", get(get_settings).post(put_settings))
        .route("/api/app/import", post(import))
        .route("/api/app/quit", post(quit))
        .route("/api/app/night/run", post(night_run))
        .route("/api/app/github", get(github_status))
        .route("/api/app/github/connect", post(github_connect))
        .route("/api/app/github/poll", post(github_poll))
        .route("/api/app/github/share", post(github_share))
        .route("/api/app/github/push", post(github_push))
        .route("/api/app/github/pages", post(github_pages))
        .route("/api/app/sync", post(sync))
        .route("/api/app/agent/check", post(agent_check))
        .route("/api/residents", get(residents_list).post(resident_spawn))
        .route("/api/residents/fund", post(residents_fund))
        .route("/api/residents/{id}", get(resident_get).delete(resident_retire))
        .route("/api/residents/{id}/grant", post(resident_grant))
        .route("/api/residents/{id}/tell", post(resident_tell))
        .route("/api/residents/{id}/think", post(resident_think))
        .route("/api/worlds/{w}/log", get(world_log).post(post_world_log))
        .route("/api/worlds/{w}/rooms/{x}/{z}", get(room).patch(patch_room))
        .route("/api/worlds/{w}/rooms/{x}/{z}/log", post(post_room_log))
        .route("/api/worlds/{w}/rooms/{x}/{z}/claim", post(claim))
        .route("/api/worlds/{w}/rooms/{x}/{z}/portals", post(add_portal))
        .route("/api/worlds/{w}/rooms/{x}/{z}/portals/{id}", axum::routing::delete(remove_portal))
        .route("/api/worlds/{w}/rooms/{x}/{z}/features", post(add_feature))
        .route("/api/players/{pid}/inbox", get(inbox))
        .route("/api/players/{pid}/inbox/read", post(inbox_read))
}

/// Credentials come from `Authorization: Bearer <player_id>:<secret>` (for
/// agents) or from `player`/`secret` fields in the JSON body (for the client).
#[derive(Deserialize, Default)]
struct Creds {
    #[serde(default)]
    player: Option<String>,
    #[serde(default)]
    secret: Option<String>,
}

enum Who {
    Player(String, String),
    Admin,
}

fn who(app: &App, uni: &Universe, headers: &HeaderMap, creds: &Creds) -> Result<Who, ApiError> {
    let bearer = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::to_string);
    if let (Some(key), Some(b)) = (&app.cfg.admin_key, &bearer) {
        if b == key {
            return Ok(Who::Admin);
        }
    }
    let (pid, secret) = match (bearer.as_deref().and_then(|b| b.split_once(':')), &creds.player, &creds.secret) {
        (Some((p, s)), _, _) => (p.to_string(), s.to_string()),
        (None, Some(p), Some(s)) => (p.clone(), s.clone()),
        _ => return Err(err(StatusCode::UNAUTHORIZED, "credentials required")),
    };
    let p = uni.auth(&pid, &secret).ok_or_else(|| err(StatusCode::UNAUTHORIZED, "unknown player or bad secret; connect to a world once first"))?;
    Ok(Who::Player(p.id.clone(), p.name.clone()))
}

fn need_world(uni: &Universe, w: &str) -> Result<(), ApiError> {
    if uni.worlds.contains_key(w) { Ok(()) } else { Err(err(StatusCode::NOT_FOUND, "no such world")) }
}

async fn host_manifest(State(app): AppState) -> Json<Value> {
    let uni = app.uni.lock().unwrap();
    Json(json!({
        "protocol": "branches/0.1",
        "software": "branches-server",
        "app": app.cfg.app,
        "version": env!("CARGO_PKG_VERSION"),
        "generators": ["liminal-houses@1"],
        "worlds": uni.worlds.values().map(|w| json!({ "id": w.manifest.id, "name": w.manifest.name, "path": format!("/w/{}/{}", w.manifest.id, w.manifest.spawn) })).collect::<Vec<_>>(),
        "endpoints": {
            "world": "/api/worlds/{world}",
            "chunk": "/api/worlds/{world}/chunk?x0&z0&x1&z1",
            "room": "/api/worlds/{world}/rooms/{x}/{z}",
            "live": "/ws/{world}",
        }
    }))
}

async fn themes() -> Json<Value> {
    Json(json!({
        "tags": procgen::TAGS,
        "themes": procgen::THEMES.iter().map(|t| json!({ "id": t.id, "name": t.name, "tags": t.tags })).collect::<Vec<_>>(),
    }))
}

async fn worlds(State(app): AppState) -> Json<Value> {
    let uni = app.uni.lock().unwrap();
    let presence = app.presence.lock().unwrap();
    Json(json!(uni.worlds.values().map(|w| json!({
        "id": w.manifest.id,
        "name": w.manifest.name,
        "tagline": w.manifest.tagline,
        "online": presence.get(&w.manifest.id).map(|p| p.len()).unwrap_or(0),
        "rooms_grown": w.rooms.values().filter(|r| r.growth() > 0).count(),
        "portal_policy": w.manifest.portal_policy,
    })).collect::<Vec<_>>()))
}

/// The world, plus every tagged version with the biome it had then, so a
/// client can draw older bands of the map the way they used to look.
async fn world(State(app): AppState, Path(w): Path<String>) -> Res {
    let uni = app.uni.lock().unwrap();
    let world = uni.worlds.get(&w).ok_or_else(|| err(StatusCode::NOT_FOUND, "no such world"))?;
    let versions: Vec<Value> = world
        .versions
        .iter()
        .map(|v| {
            let biome = app.store.world_at(&v.tag, &w).map(|m| m.generator.params).unwrap_or(Value::Null);
            json!({ "tag": v.tag, "at": v.at, "summary": v.summary, "biome": biome })
        })
        .collect();
    Ok(Json(json!({ "manifest": world.manifest, "versions": versions, "ring": crate::state::RING })))
}

/// The world graph as this host knows it: worlds are nodes, portals edges.
async fn graph(State(app): AppState) -> Json<Value> {
    let uni = app.uni.lock().unwrap();
    let mut edges = Vec::new();
    for w in uni.worlds.values() {
        for r in w.rooms.values() {
            for p in &r.portals {
                edges.push(json!({ "from": format!("/w/{}/{},{}", w.manifest.id, r.x, r.z), "to": p.target, "label": p.label, "by": p.by }));
            }
        }
    }
    Json(json!({
        "nodes": uni.worlds.values().map(|w| json!({ "id": w.manifest.id, "name": w.manifest.name, "policy": w.manifest.portal_policy.mode })).collect::<Vec<_>>(),
        "edges": edges,
        "note": "Untouched rooms also carry generator-placed doors; only stored rooms are listed.",
    }))
}

#[derive(Deserialize)]
struct ChunkQ {
    x0: i32,
    z0: i32,
    x1: i32,
    z1: i32,
}

async fn chunk(State(app): AppState, Path(w): Path<String>, Query(q): Query<ChunkQ>) -> Res {
    let uni = app.uni.lock().unwrap();
    need_world(&uni, &w)?;
    let (x0, x1) = (q.x0.min(q.x1), q.x0.max(q.x1));
    let (z0, z1) = (q.z0.min(q.z1), q.z0.max(q.z1));
    if (x1 - x0 + 1) as i64 * (z1 - z0 + 1) as i64 > 625 {
        return Err(err(StatusCode::BAD_REQUEST, "chunk too large (max 625 cells)"));
    }
    let world = &uni.worlds[&w];
    let mut cells = Vec::new();
    for x in x0..=x1 {
        for z in z0..=z1 {
            if let Some((r, _)) = app.displayed(&uni, &w, x, z) {
                let mut s = room_summary(&r);
                s["band"] = json!(app.band(world, x, z).0);
                if let Some(cur) = world.rooms.get(&addr(x, z)) {
                    s["building"] = json!(cur.building.is_some());
                }
                cells.push(s);
            }
        }
    }
    Ok(Json(json!({ "cells": cells })))
}

async fn room(State(app): AppState, Path((w, x, z)): Path<(String, i32, i32)>) -> Res {
    let uni = app.uni.lock().unwrap();
    room_view(&app, &uni, &w, x, z).map(Json).ok_or_else(|| err(StatusCode::NOT_FOUND, "no such world"))
}

/// What has been explored: heat per cell, grown houses and doors, for the
/// map people carry and the heat map architects read.
async fn map(State(app): AppState, Path(w): Path<String>) -> Res {
    let uni = app.uni.lock().unwrap();
    let world = uni.worlds.get(&w).ok_or_else(|| err(StatusCode::NOT_FOUND, "no such world"))?;
    let heat: Vec<Value> = world.heat.iter().filter_map(|(k, h)| parse_addr(k).map(|(x, z)| json!([x, z, h.inside.round(), h.outside.round(), h.visits]))).collect();
    let rooms: Vec<Value> = world
        .rooms
        .values()
        .filter(|r| r.growth() > 0 || !r.things.is_empty() || r.portals.iter().any(|p| p.by != "generator"))
        .map(|r| json!({ "x": r.x, "z": r.z, "growth": r.growth(), "things": r.things.len(), "doors": r.portals.iter().map(|p| p.label.clone()).collect::<Vec<_>>(), "building": r.building.is_some() }))
        .collect();
    Ok(Json(json!({ "spawn": world.manifest.spawn, "heat": heat, "rooms": rooms, "versions": world.versions.len(), "ring": crate::state::RING })))
}

#[derive(Deserialize)]
struct LogBody {
    #[serde(flatten)]
    creds: Creds,
    kind: Option<String>,
    text: String,
}

fn clean(text: &str, max: usize) -> Result<String, ApiError> {
    let t: String = text.trim().chars().filter(|c| !c.is_control()).take(max).collect();
    if t.is_empty() { Err(err(StatusCode::BAD_REQUEST, "empty text")) } else { Ok(t) }
}

async fn post_room_log(State(app): AppState, Path((w, x, z)): Path<(String, i32, i32)>, headers: HeaderMap, Json(b): Json<LogBody>) -> Res {
    let text = clean(&b.text, 400)?;
    let kind = match b.kind.as_deref() {
        Some("request") => "request",
        Some("praise") => "praise",
        _ => "note",
    };
    let mut uni = app.uni.lock().unwrap();
    need_world(&uni, &w)?;
    let (pid, name) = match who(&app, &uni, &headers, &b.creds)? {
        Who::Player(p, n) => (Some(p), n),
        Who::Admin => (None, "host".into()),
    };
    let room = uni.room_mut(&w, x, z).unwrap();
    let heard = if kind == "request" { architect::record_request(room, &text) } else { vec![] };
    let entry = LogEntry { id: new_id(), kind: kind.into(), who: name, player: pid, text, at: now_ms() };
    push_log(&mut room.log, entry.clone());
    app.touch();
    drop(uni);
    app.broadcast(&w, &json!({ "t": "log", "addr": addr(x, z), "entry": entry }));
    Ok(Json(json!({ "ok": true, "entry": entry, "architect_heard": heard })))
}

async fn world_log(State(app): AppState, Path(w): Path<String>) -> Res {
    let uni = app.uni.lock().unwrap();
    let world = uni.worlds.get(&w).ok_or_else(|| err(StatusCode::NOT_FOUND, "no such world"))?;
    Ok(Json(json!({ "log": world.log.iter().rev().take(100).collect::<Vec<_>>() })))
}

async fn post_world_log(State(app): AppState, Path(w): Path<String>, headers: HeaderMap, Json(b): Json<LogBody>) -> Res {
    let text = clean(&b.text, 400)?;
    let mut uni = app.uni.lock().unwrap();
    need_world(&uni, &w)?;
    let (pid, name) = match who(&app, &uni, &headers, &b.creds)? {
        Who::Player(p, n) => (Some(p), n),
        Who::Admin => (None, "host".into()),
    };
    let entry = LogEntry { id: new_id(), kind: b.kind.unwrap_or_else(|| "note".into()), who: name, player: pid, text, at: now_ms() };
    push_log(&mut uni.worlds.get_mut(&w).unwrap().log, entry.clone());
    app.touch();
    Ok(Json(json!({ "ok": true, "entry": entry })))
}

#[derive(Deserialize)]
struct ClaimBody {
    #[serde(flatten)]
    creds: Creds,
    title: Option<String>,
}

async fn claim(State(app): AppState, Path((w, x, z)): Path<(String, i32, i32)>, headers: HeaderMap, Json(b): Json<ClaimBody>) -> Res {
    let mut uni = app.uni.lock().unwrap();
    need_world(&uni, &w)?;
    let Who::Player(pid, name) = who(&app, &uni, &headers, &b.creds)? else {
        return Err(err(StatusCode::BAD_REQUEST, "claims belong to players"));
    };
    let owned = uni.worlds[&w].rooms.values().filter(|r| r.claim.as_ref().is_some_and(|c| c.owner == pid)).count();
    if owned >= 3 {
        return Err(err(StatusCode::FORBIDDEN, "you already tend three addresses in this world"));
    }
    let title = clean(b.title.as_deref().unwrap_or(&format!("{name}'s house")), 60)?;
    let room = uni.room_mut(&w, x, z).unwrap();
    if room.claim.is_some() {
        return Err(err(StatusCode::CONFLICT, "already claimed"));
    }
    room.claim = Some(Claim { owner: pid.clone(), owner_name: name.clone(), title: title.clone(), since: now_ms(), outbound: OutboundRule::default() });
    let entry = LogEntry { id: new_id(), kind: "claim".into(), who: name, player: Some(pid), text: format!("claimed this address as \u{201c}{title}\u{201d}"), at: now_ms() };
    push_log(&mut room.log, entry);
    let summary = room_summary(room);
    app.touch();
    drop(uni);
    app.broadcast(&w, &json!({ "t": "room", "addr": addr(x, z), "summary": summary }));
    Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
struct PatchBody {
    #[serde(flatten)]
    creds: Creds,
    title: Option<String>,
    theme: Option<String>,
    outbound: Option<OutboundRule>,
}

fn require_owner(who: &Who, room: &Room) -> Result<(), ApiError> {
    match who {
        Who::Admin => Ok(()),
        Who::Player(pid, _) if room.claim.as_ref().is_some_and(|c| &c.owner == pid) => Ok(()),
        _ => Err(err(StatusCode::FORBIDDEN, "only the address's owner can do that; claim it first")),
    }
}

async fn patch_room(State(app): AppState, Path((w, x, z)): Path<(String, i32, i32)>, headers: HeaderMap, Json(b): Json<PatchBody>) -> Res {
    let mut uni = app.uni.lock().unwrap();
    need_world(&uni, &w)?;
    let who = who(&app, &uni, &headers, &b.creds)?;
    let room = uni.room_mut(&w, x, z).unwrap();
    require_owner(&who, room)?;
    if let Some(t) = &b.theme {
        if procgen::theme(t).is_none() {
            return Err(err(StatusCode::BAD_REQUEST, "unknown theme; see /api/themes"));
        }
        room.theme = t.clone();
    }
    if let Some(c) = room.claim.as_mut() {
        if let Some(t) = &b.title {
            c.title = clean(t, 60)?;
        }
        if let Some(o) = b.outbound {
            c.outbound = o;
        }
    }
    let summary = room_summary(room);
    app.touch();
    drop(uni);
    app.broadcast(&w, &json!({ "t": "room", "addr": addr(x, z), "summary": summary, "rebuild": true }));
    Ok(Json(json!({ "ok": true })))
}

/// `/w/<world>/<x>,<z>` on this host.
fn local_target(target: &str) -> Option<(String, i32, i32)> {
    let rest = target.strip_prefix("/w/")?;
    let (w, a) = rest.split_once('/')?;
    let (x, z) = parse_addr(a)?;
    Some((w.to_string(), x, z))
}

fn host_of(target: &str) -> Option<String> {
    let rest = target.strip_prefix("https://").or_else(|| target.strip_prefix("http://"))?;
    let host = rest.split('/').next()?.to_lowercase();
    (!host.is_empty()).then_some(host)
}

/// Check a new portal against the world policy and the owner's own rule.
fn check_portal(policy: &PortalPolicy, room: &Room, target: &str) -> Result<(), ApiError> {
    let local = target.starts_with("/w/");
    let host = host_of(target);
    if !local && host.is_none() {
        return Err(err(StatusCode::BAD_REQUEST, "target must be /w/<world>/<x>,<z> or an http(s) URL"));
    }
    let deny = |m: &str| Err(err(StatusCode::FORBIDDEN, m.to_string()));
    if policy.mode == PolicyMode::Closed {
        return deny("this world is closed to new outbound portals");
    }
    let built = room.portals.iter().filter(|p| p.by != "generator" && p.by != "return").count() as u32;
    if built >= policy.max_per_room {
        return deny("this room has reached the world's portal limit");
    }
    if policy.mode == PolicyMode::Allowlist {
        let ok = if local { policy.allow_local } else { host.as_ref().is_some_and(|h| policy.allow_hosts.contains(h)) };
        if !ok {
            return deny("the world's allowlist does not include that destination");
        }
    }
    if let Some(rule) = room.claim.as_ref().map(|c| &c.outbound) {
        if rule.closed {
            return deny("the owner has closed this address to new portals");
        }
        if rule.max.is_some_and(|m| built >= m) {
            return deny("the owner's portal limit is reached");
        }
        if let (Some(hosts), Some(h)) = (&rule.allow_hosts, &host) {
            if !hosts.contains(h) {
                return deny("the owner's allowlist does not include that host");
            }
        }
    }
    Ok(())
}

#[derive(Deserialize)]
struct PortalBody {
    #[serde(flatten)]
    creds: Creds,
    target: String,
    label: Option<String>,
    slot: Option<u32>,
}

async fn add_portal(State(app): AppState, Path((w, x, z)): Path<(String, i32, i32)>, headers: HeaderMap, Json(b): Json<PortalBody>) -> Res {
    let target = clean(&b.target, 300)?;
    let mut uni = app.uni.lock().unwrap();
    need_world(&uni, &w)?;
    if let Some(rest) = target.strip_prefix("/w/") {
        let world_id = rest.split('/').next().unwrap_or("");
        if !uni.worlds.contains_key(world_id) {
            return Err(err(StatusCode::BAD_REQUEST, "no such local world"));
        }
    }
    let who = who(&app, &uni, &headers, &b.creds)?;
    let policy = uni.worlds[&w].manifest.portal_policy.clone();
    let room = uni.room_mut(&w, x, z).unwrap();
    require_owner(&who, room)?;
    if !matches!(who, Who::Admin) {
        check_portal(&policy, room, &target)?;
    }
    let label = clean(b.label.as_deref().unwrap_or("Somewhere else"), 60)?;
    let by = match &who {
        Who::Player(_, n) => n.clone(),
        Who::Admin => "host".into(),
    };
    let slot = b.slot.unwrap_or(0).min(room.growth());
    let portal = Portal { id: new_id(), slot, target: target.clone(), label: label.clone(), by: by.clone(), at: now_ms(), sealed: false };
    room.portals.push(portal.clone());
    push_log(&mut room.log, LogEntry { id: new_id(), kind: "portal".into(), who: by, player: None, text: format!("opened a door to {label} ({target})"), at: now_ms() });
    // Doors are physical: a local destination gets the door back in its
    // entry hall (sealed if that world is closed to doors out).
    let back = local_target(&target).and_then(|(tw, tx, tz)| {
        let name = uni.worlds[&w].manifest.name.clone();
        let mode = uni.worlds.get(&tw)?.manifest.portal_policy.mode;
        let r = uni.room_mut(&tw, tx, tz)?;
        r.portals.push(Portal {
            id: format!("ret-{}", portal.id),
            slot: 0,
            target: format!("/w/{w}/{x},{z}"),
            label: name,
            by: "return".into(),
            at: now_ms(),
            sealed: mode == PolicyMode::Closed,
        });
        Some((tw, addr(tx, tz)))
    });
    app.touch();
    drop(uni);
    app.broadcast(&w, &json!({ "t": "room", "addr": addr(x, z), "rebuild": true }));
    if let Some((tw, a)) = back {
        app.broadcast(&tw, &json!({ "t": "room", "addr": a, "rebuild": true }));
    }
    Ok(Json(json!({ "ok": true, "portal": portal })))
}

async fn remove_portal(State(app): AppState, Path((w, x, z, id)): Path<(String, i32, i32, String)>, headers: HeaderMap, body: Option<Json<Creds>>) -> Res {
    let creds = body.map(|b| b.0).unwrap_or_default();
    let mut uni = app.uni.lock().unwrap();
    need_world(&uni, &w)?;
    let who = who(&app, &uni, &headers, &creds)?;
    let room = uni.room_mut(&w, x, z).unwrap();
    require_owner(&who, room)?;
    let Some(gone) = room.portals.iter().position(|p| p.id == id).map(|i| room.portals.remove(i)) else {
        return Err(err(StatusCode::NOT_FOUND, "no such portal"));
    };
    let back = local_target(&gone.target).and_then(|(tw, tx, tz)| {
        let r = uni.worlds.get_mut(&tw)?.rooms.get_mut(&addr(tx, tz))?;
        r.portals.retain(|p| p.id != format!("ret-{id}"));
        Some((tw, addr(tx, tz)))
    });
    app.touch();
    drop(uni);
    app.broadcast(&w, &json!({ "t": "room", "addr": addr(x, z), "rebuild": true }));
    if let Some((tw, a)) = back {
        app.broadcast(&tw, &json!({ "t": "room", "addr": a, "rebuild": true }));
    }
    Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
struct FeatureBody {
    #[serde(flatten)]
    creds: Creds,
    tag: String,
    chamber: Option<u32>,
}

/// Owners' agents can build directly, alongside the architect.
async fn add_feature(State(app): AppState, Path((w, x, z)): Path<(String, i32, i32)>, headers: HeaderMap, Json(b): Json<FeatureBody>) -> Res {
    if !procgen::TAGS.contains(&b.tag.as_str()) {
        return Err(err(StatusCode::BAD_REQUEST, "unknown tag; see /api/themes"));
    }
    let mut uni = app.uni.lock().unwrap();
    need_world(&uni, &w)?;
    let who = who(&app, &uni, &headers, &b.creds)?;
    let room = uni.room_mut(&w, x, z).unwrap();
    require_owner(&who, room)?;
    if room.features.len() >= 200 {
        return Err(err(StatusCode::FORBIDDEN, "this room is full"));
    }
    let chamber = b.chamber.unwrap_or(room.growth()).min(room.growth());
    let by = match &who {
        Who::Player(_, n) => format!("agent:{n}"),
        Who::Admin => "host".into(),
    };
    let f = Feature { id: new_id(), tag: b.tag, chamber, seed: (procgen::splitmix(now_ms()) & 0xffff_ffff) as u32, by, at: now_ms() };
    room.features.push(f.clone());
    app.touch();
    drop(uni);
    app.broadcast(&w, &json!({ "t": "room", "addr": addr(x, z), "rebuild": true }));
    Ok(Json(json!({ "ok": true, "feature": f })))
}

#[derive(Deserialize)]
struct SecretQ {
    secret: String,
}

async fn inbox(State(app): AppState, Path(pid): Path<String>, Query(q): Query<SecretQ>) -> Res {
    let uni = app.uni.lock().unwrap();
    let p = uni.auth(&pid, &q.secret).ok_or_else(|| err(StatusCode::UNAUTHORIZED, "bad credentials"))?;
    Ok(Json(json!({ "inbox": p.inbox.iter().rev().collect::<Vec<_>>(), "prefs": p.prefs })))
}

async fn inbox_read(State(app): AppState, Path(pid): Path<String>, Json(q): Json<SecretQ>) -> Res {
    let mut uni = app.uni.lock().unwrap();
    if uni.auth(&pid, &q.secret).is_none() {
        return Err(err(StatusCode::UNAUTHORIZED, "bad credentials"));
    }
    uni.players.get_mut(&pid).unwrap().inbox.iter_mut().for_each(|n| n.read = true);
    app.touch();
    Ok(Json(json!({ "ok": true })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::procgen;

    fn room() -> Room {
        let m = procgen::builtin_worlds().remove(0);
        procgen::default_room(&m, &[], 3, 3)
    }

    #[test]
    fn closed_worlds_refuse_portals() {
        let p = PortalPolicy { mode: PolicyMode::Closed, max_per_room: 9, allow_hosts: vec![], allow_local: true };
        assert!(check_portal(&p, &room(), "/w/the-lush/0,0").is_err());
    }

    #[test]
    fn allowlist_checks_hosts() {
        let p = PortalPolicy { mode: PolicyMode::Allowlist, max_per_room: 9, allow_hosts: vec!["friend.example".into()], allow_local: true };
        let r = room();
        assert!(check_portal(&p, &r, "/w/the-lush/1,1").is_ok());
        assert!(check_portal(&p, &r, "https://friend.example/w/x/0,0").is_ok());
        assert!(check_portal(&p, &r, "https://stranger.example/w/x/0,0").is_err());
        assert!(check_portal(&p, &r, "javascript:alert(1)").is_err());
    }

    #[test]
    fn owners_can_only_tighten() {
        let p = PortalPolicy { mode: PolicyMode::Open, max_per_room: 4, allow_hosts: vec![], allow_local: true };
        let mut r = room();
        r.claim = Some(Claim { owner: "a".into(), owner_name: "a".into(), title: "t".into(), since: 0, outbound: OutboundRule { closed: true, ..Default::default() } });
        assert!(check_portal(&p, &r, "/w/the-lush/1,1").is_err());
        r.claim.as_mut().unwrap().outbound = OutboundRule { max: Some(0), ..Default::default() };
        assert!(check_portal(&p, &r, "/w/the-lush/1,1").is_err());
    }
}

// ------------------------------------------------------------ the desktop app

async fn get_settings(State(app): AppState) -> Json<Value> {
    Json(json!({
        "app": app.cfg.app,
        "settings": *app.settings.lock().unwrap(),
        "data_dir": app.store.root,
        "night_running": app.night_running.load(std::sync::atomic::Ordering::Relaxed),
        "night_built": crate::nightshift::built(&app),
        "claude": crate::agent::find_claude().map(|p| p.to_string_lossy().to_string()),
    }))
}

/// Settings choose a command the app will run, so only the app's own page
/// may change them: never another website open in the same browser.
fn same_origin(app: &App, headers: &HeaderMap) -> bool {
    match headers.get("origin").and_then(|v| v.to_str().ok()) {
        None => true,
        Some(o) => o == format!("http://localhost:{}", app.cfg.port) || o == format!("http://127.0.0.1:{}", app.cfg.port),
    }
}

async fn put_settings(State(app): AppState, headers: HeaderMap, Json(s): Json<crate::state::Settings>) -> Res {
    if !same_origin(&app, &headers) {
        return Err(err(StatusCode::FORBIDDEN, "settings can only be changed from the app's own page"));
    }
    if !app.cfg.app {
        return Err(err(StatusCode::FORBIDDEN, "settings can only be changed in the desktop app"));
    }
    if !["heuristic", "command"].contains(&s.architect.as_str()) {
        return Err(err(StatusCode::BAD_REQUEST, "architect must be heuristic or command"));
    }
    // Saving settings is choosing: don't ask who should build again.
    let mut s = s;
    s.asked = true;
    app.save_settings(s);
    Ok(Json(json!({ "ok": true })))
}

async fn quit(State(app): AppState, headers: HeaderMap) -> Res {
    if !same_origin(&app, &headers) {
        return Err(err(StatusCode::FORBIDDEN, "the app can only be quit from its own page"));
    }
    if !app.cfg.app {
        return Err(err(StatusCode::FORBIDDEN, "only the desktop app can be quit from the page"));
    }
    app.quit.notify_one();
    Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
struct ImportBody {
    #[serde(default)]
    rooms: BTreeMap<String, BTreeMap<String, Value>>,
    #[serde(default)]
    player: Option<Value>,
}

/// Bring worlds grown in a browser (with no app running) into the app.
/// Houses the app already knows are kept; new ones are added.
async fn import(State(app): AppState, Json(b): Json<ImportBody>) -> Res {
    let mut uni = app.uni.lock().unwrap();
    let mut added = 0;
    for (wid, rooms) in b.rooms {
        let Some(world) = uni.worlds.get_mut(&wid) else { continue };
        for (key, v) in rooms {
            let Ok(room) = serde_json::from_value::<Room>(v) else { continue };
            if addr(room.x, room.z) != key || world.rooms.contains_key(&key) {
                continue;
            }
            world.rooms.insert(key, room);
            added += 1;
        }
    }
    if let Some(p) = b.player.and_then(|v| serde_json::from_value::<Player>(v).ok()) {
        uni.players.entry(p.id.clone()).or_insert(p);
    }
    drop(uni);
    if added > 0 {
        app.note_commit(format!("Imported {added} houses grown in a browser"));
        app.touch();
    }
    Ok(Json(json!({ "ok": true, "rooms": added })))
}

// ------------------------------------------------------------ GitHub

fn app_only(app: &App, headers: &HeaderMap) -> Result<(), ApiError> {
    if !app.cfg.app || !same_origin(app, headers) {
        return Err(err(StatusCode::FORBIDDEN, "only the app's own page can do that"));
    }
    Ok(())
}

async fn github_status(State(app): AppState) -> Json<Value> {
    let root = app.store.root.clone();
    let a = crate::github::load(&root);
    Json(json!({
        "connected": a.is_some(),
        "login": a.as_ref().map(|a| a.login.clone()),
        "fork": a.as_ref().and_then(|a| a.fork.clone()),
        "shared": a.as_ref().and_then(|a| a.shared.clone()),
        "push_error": a.as_ref().and_then(|a| a.push_error.clone()),
        "page": a.as_ref().and_then(|a| a.page.clone()),
        "share_link": a.as_ref().and_then(|a| a.fork.as_deref().map(crate::pages::share_link)),
        "last_push": a.as_ref().map(|a| a.last_push),
        "device": crate::github::client_id().is_some(),
    }))
}

#[derive(Deserialize, Default)]
struct ConnectBody {
    #[serde(default)]
    token: Option<String>,
    #[serde(default)]
    device_code: Option<String>,
}

/// Sign in with a token, fork, bring back worlds already on the fork (a new
/// Mac, a reinstall), push. Shared by every way of connecting.
fn finish_connect(app: &App, token: String) -> Result<Value, String> {
    let root = app.store.root.clone();
    let login = crate::github::whoami(&token)?;
    let fork = crate::github::fork(&token)?;
    let mut a = crate::github::Account { token, login: login.clone(), fork: Some(fork.clone()), ..Default::default() };
    if let Some(old) = crate::github::load(&root) {
        a.shared = old.shared;
    }
    crate::github::save(&root, &a);
    let restored = match crate::sync::sync(app, crate::sync::Source::Fork) {
        Ok(r) => r.files,
        Err(e) => {
            tracing::warn!("could not bring worlds back from {fork}: {e}");
            0
        }
    };
    let pushed = crate::github::push_patiently(&root, &a);
    a.last_push = now_ms();
    a.push_error = pushed.clone().err();
    crate::github::save(&root, &a);
    Ok(json!({ "connected": true, "login": login, "fork": fork, "pushed": pushed.is_ok(), "push_error": a.push_error, "restored": restored, "shared": a.shared }))
}

async fn github_connect(State(app): AppState, headers: HeaderMap, body: Option<Json<ConnectBody>>) -> Res {
    app_only(&app, &headers)?;
    let token = body.and_then(|b| b.0.token).map(|t| t.trim().to_string()).filter(|t| !t.is_empty());
    let out = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        if let Some(t) = token.or_else(crate::github::gh_cli_token) {
            return finish_connect(&app, t);
        }
        if crate::github::client_id().is_some() {
            return Ok(json!({ "device": crate::github::device_start()? }));
        }
        Ok(json!({ "needs_token": true }))
    })
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    out.map(Json).map_err(|e| err(StatusCode::BAD_GATEWAY, e))
}

async fn github_poll(State(app): AppState, headers: HeaderMap, Json(b): Json<ConnectBody>) -> Res {
    app_only(&app, &headers)?;
    let code = b.device_code.ok_or_else(|| err(StatusCode::BAD_REQUEST, "device_code required"))?;
    let out = tokio::task::spawn_blocking(move || match crate::github::device_poll(&code)? {
        Some(token) => finish_connect(&app, token),
        None => Ok(json!({ "waiting": true })),
    })
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    out.map(Json).map_err(|e| err(StatusCode::BAD_GATEWAY, e))
}

#[derive(Deserialize, Default)]
struct ShareBody {
    #[serde(flatten)]
    creds: Creds,
    /// Share one house (claimed or not, grown or empty).
    world: Option<String>,
    x: Option<i32>,
    z: Option<i32>,
}

/// Share on GitHub: one house, or your worlds. Nothing needs to have grown:
/// an empty house is a fine place to start.
async fn github_share(State(app): AppState, headers: HeaderMap, body: Option<Json<ShareBody>>) -> Res {
    app_only(&app, &headers)?;
    let b = body.map(|b| b.0).unwrap_or_default();
    let root = app.store.root.clone();
    let mut a = crate::github::load(&root).ok_or_else(|| err(StatusCode::BAD_REQUEST, "connect GitHub first"))?;
    let (worlds, rooms) = {
        let mut uni = app.uni.lock().unwrap();
        match (b.world, b.x, b.z) {
            (Some(w), Some(x), Some(z)) => {
                need_world(&uni, &w)?;
                let name = match who(&app, &uni, &headers, &b.creds) {
                    Ok(Who::Player(_, n)) => n,
                    _ => "someone".into(),
                };
                // Store the house so it is in the repository, even untouched.
                let room = uni.room_mut(&w, x, z).ok_or_else(|| err(StatusCode::NOT_FOUND, "no such house"))?;
                let title = room.claim.as_ref().map(|c| c.title.clone()).unwrap_or_else(|| format!("{name}'s house"));
                (vec![], vec![crate::github::SharedRoom { world: w, x, z, title }])
            }
            _ => {
                // Born worlds first; then whatever grew; else every world.
                let mut ws: Vec<&World> = uni.worlds.values().filter(|w| !w.manifest.hub || w.rooms.values().any(|r| r.growth() > 0 || r.claim.is_some())).collect();
                if ws.is_empty() {
                    ws = uni.worlds.values().collect();
                }
                (ws.iter().map(|w| (w.manifest.id.clone(), w.manifest.name.clone())).collect(), vec![])
            }
        }
    };
    app.touch();
    let out = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        {
            let uni = app.uni.lock().unwrap();
            app.store.save(&uni).map_err(|e| e.to_string())?;
            app.store.commit(&format!("Share {}", rooms.first().map(|r| r.title.clone()).unwrap_or_else(|| "worlds".into()))).map_err(|e| e.to_string())?;
        }
        // The door must lead somewhere: the fork has to have the worlds first.
        a.last_push = now_ms();
        if let Err(e) = crate::github::push_patiently(&root, &a) {
            a.push_error = Some(e.clone());
            crate::github::save(&root, &a);
            return Err(format!("could not back up to {}: {e}", a.fork.clone().unwrap_or_default()));
        }
        a.push_error = None;
        let url = crate::github::share(&a, &worlds, &rooms)?;
        a.shared = Some(url.clone());
        crate::github::save(&root, &a);
        Ok(json!({ "url": url, "rooms": rooms, "worlds": worlds.len() }))
    })
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    out.map(Json).map_err(|e| err(StatusCode::BAD_GATEWAY, e))
}

#[derive(Deserialize)]
struct CheckBody {
    command: String,
}

/// Try an architect command before using it: installed, signed in, answering?
async fn agent_check(State(app): AppState, headers: HeaderMap, Json(b): Json<CheckBody>) -> Res {
    app_only(&app, &headers)?;
    let root = app.store.root.clone();
    let r = tokio::task::spawn_blocking(move || crate::agent::check(&b.command, &root)).await.map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(match r {
        Ok(took) => json!({ "ok": true, "seconds": took.as_secs_f32() }),
        Err(e) => json!({ "ok": false, "error": e }),
    }))
}

/// Your own page on GitHub Pages: an entry point to your worlds to share.
async fn github_pages(State(app): AppState, headers: HeaderMap) -> Res {
    app_only(&app, &headers)?;
    let out = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let root = app.store.root.clone();
        let mut a = crate::github::load(&root).ok_or("connect GitHub first")?;
        // The page reads your worlds from the fork, so back up first.
        {
            let uni = app.uni.lock().unwrap();
            app.store.save(&uni).map_err(|e| e.to_string())?;
            app.store.commit("Back up").map_err(|e| e.to_string())?;
        }
        crate::github::push_patiently(&root, &a)?;
        let (url, share) = crate::pages::publish(&root, &app.cfg.client_dir, &a)?;
        a.page = Some(url.clone());
        crate::github::save(&root, &a);
        Ok(json!({ "url": url, "share": share }))
    })
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    out.map(Json).map_err(|e| err(StatusCode::BAD_GATEWAY, e))
}

/// Back up now.
async fn github_push(State(app): AppState, headers: HeaderMap) -> Res {
    app_only(&app, &headers)?;
    let out = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let root = app.store.root.clone();
        let mut a = crate::github::load(&root).ok_or("connect GitHub first")?;
        {
            let uni = app.uni.lock().unwrap();
            app.store.save(&uni).map_err(|e| e.to_string())?;
            app.store.commit("Back up").map_err(|e| e.to_string())?;
        }
        let r = crate::github::push_patiently(&root, &a);
        a.last_push = now_ms();
        a.push_error = r.clone().err();
        crate::github::save(&root, &a);
        r.map(|_| json!({ "ok": true }))
    })
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    out.map(Json).map_err(|e| err(StatusCode::BAD_GATEWAY, e))
}

#[derive(Deserialize, Default)]
struct SyncBody {
    /// "upstream" (the main world, the default) or "fork" (your own backup).
    #[serde(default)]
    from: Option<String>,
}

/// Bring in changes from the main world (or your fork). Houses changed on
/// both sides are merged by your agent when the architect is a command.
async fn sync(State(app): AppState, headers: HeaderMap, body: Option<Json<SyncBody>>) -> Res {
    app_only(&app, &headers)?;
    let from = body.and_then(|b| b.0.from).unwrap_or_else(|| "upstream".into());
    if app.syncing.swap(true, std::sync::atomic::Ordering::Relaxed) {
        return Err(err(StatusCode::CONFLICT, "already syncing"));
    }
    let a2 = app.clone();
    let out = tokio::task::spawn_blocking(move || -> Result<crate::sync::Report, String> {
        let source = if from == "fork" { crate::sync::Source::Fork } else { crate::sync::Source::Upstream };
        let upstream = matches!(source, crate::sync::Source::Upstream);
        let mut rep = crate::sync::sync(&a2, source)?;
        if upstream {
            if let Some(acct) = crate::github::load(&a2.store.root) {
                rep.fork_code = Some(crate::github::sync_fork_code(&acct).unwrap_or_else(|e| format!("could not update your fork's code: {e}")));
            }
        }
        if !rep.up_to_date {
            a2.note_commit(format!("Synced with {}", rep.source));
        }
        Ok(rep)
    })
    .await;
    app.syncing.store(false, std::sync::atomic::Ordering::Relaxed);
    let rep = out.map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?.map_err(|e| err(StatusCode::BAD_GATEWAY, e))?;
    if !rep.up_to_date {
        let ids: Vec<String> = app.uni.lock().unwrap().worlds.keys().cloned().collect();
        for w in ids {
            app.broadcast(&w, &json!({ "t": "synced" }));
        }
    }
    Ok(Json(serde_json::to_value(rep).unwrap_or_default()))
}

/// "Build something now": start a night-shift project immediately.
async fn night_run(State(app): AppState, headers: HeaderMap) -> Res {
    app_only(&app, &headers)?;
    match crate::nightshift::start(&app, true) {
        Some(title) => Ok(Json(json!({ "started": title }))),
        None => Err(err(StatusCode::CONFLICT, "a project is already being built")),
    }
}

// ------------------------------------------------------------ residents

fn resident_summary(r: &crate::residents::Resident) -> Value {
    json!({
        "id": r.id, "name": r.name, "color": r.color, "persona": r.persona, "goal": r.goal,
        "world": r.world, "room": r.room, "cell": r.cell(), "coins": r.coins, "tokens": r.tokens,
        "thoughts": r.thoughts, "said": r.said, "creations": r.creations.len(),
        "earned": r.creations.iter().map(|c| c.earned).sum::<f64>().abs(),
        "memory": r.memories.last().map(|m| m.text.clone()),
    })
}

async fn residents_list(State(app): AppState) -> Json<Value> {
    let uni = app.uni.lock().unwrap();
    let e = &uni.economy;
    Json(json!({
        "tokens_per_coin": crate::residents::TOKENS_PER_COIN,
        "economy": { "treasury": e.treasury, "added": e.added, "tokens": e.tokens, "ledger": e.ledger.iter().rev().take(40).collect::<Vec<_>>() },
        "residents": uni.residents.values().map(resident_summary).collect::<Vec<_>>(),
        "thinking": app.residents_thinking.load(std::sync::atomic::Ordering::Relaxed),
    }))
}

async fn resident_get(State(app): AppState, Path(id): Path<String>) -> Res {
    let uni = app.uni.lock().unwrap();
    let r = uni.residents.get(&id).ok_or_else(|| err(StatusCode::NOT_FOUND, "no such resident"))?;
    let mut v = serde_json::to_value(r).unwrap_or_default();
    v["cell"] = json!(r.cell());
    Ok(Json(v))
}

#[derive(Deserialize)]
struct SpawnBody {
    #[serde(flatten)]
    creds: Creds,
    name: String,
    #[serde(default)]
    persona: String,
    #[serde(default)]
    goal: String,
    world: String,
    #[serde(default)]
    x: i32,
    #[serde(default)]
    z: i32,
    #[serde(default)]
    coins: Option<f64>,
}

/// Bring a resident to life at an address, with coins from the treasury.
async fn resident_spawn(State(app): AppState, headers: HeaderMap, Json(b): Json<SpawnBody>) -> Res {
    app_only(&app, &headers)?;
    let name = clean(&b.name, 40)?;
    let mut uni = app.uni.lock().unwrap();
    if uni.residents.len() >= 24 {
        return Err(err(StatusCode::CONFLICT, "the worlds already have 24 residents"));
    }
    let by = match who(&app, &uni, &headers, &b.creds) {
        Ok(Who::Player(p, _)) => Some(p),
        _ => None,
    };
    let coins = b.coins.unwrap_or(20.0);
    let r = crate::residents::spawn(&mut uni, &name, &b.persona, &b.goal, &b.world, (b.x, b.z), coins, by).map_err(|e| err(StatusCode::BAD_REQUEST, e))?;
    drop(uni);
    app.note_commit(format!("{} moved into {} at {},{}", r.name, b.world, b.x, b.z));
    app.touch();
    Ok(Json(resident_summary(&r)))
}

#[derive(Deserialize)]
struct CoinsBody {
    coins: f64,
}

/// Add coins to the treasury: what residents may spend on thinking and building.
async fn residents_fund(State(app): AppState, headers: HeaderMap, Json(b): Json<CoinsBody>) -> Res {
    app_only(&app, &headers)?;
    if !(b.coins.is_finite() && b.coins > 0.0 && b.coins <= 100_000.0) {
        return Err(err(StatusCode::BAD_REQUEST, "add between 0 and 100,000 coins"));
    }
    let mut uni = app.uni.lock().unwrap();
    uni.economy.treasury = ((uni.economy.treasury + b.coins) * 100.0).round() / 100.0;
    uni.economy.added += b.coins;
    uni.economy.record("you", "treasury", b.coins, "added to the treasury");
    let t = uni.economy.treasury;
    drop(uni);
    app.note_commit(format!("Added {} coins to the residents' treasury", b.coins));
    app.touch();
    Ok(Json(json!({ "treasury": t })))
}

async fn resident_grant(State(app): AppState, headers: HeaderMap, Path(id): Path<String>, Json(b): Json<CoinsBody>) -> Res {
    app_only(&app, &headers)?;
    let mut uni = app.uni.lock().unwrap();
    if !uni.residents.contains_key(&id) {
        return Err(err(StatusCode::NOT_FOUND, "no such resident"));
    }
    let coins = (b.coins.max(0.0) * 100.0).round() / 100.0;
    if coins <= 0.0 || coins > uni.economy.treasury {
        return Err(err(StatusCode::BAD_REQUEST, format!("the treasury has {:.2} coins", uni.economy.treasury)));
    }
    uni.economy.treasury -= coins;
    let r = uni.residents.get_mut(&id).unwrap();
    r.coins += coins;
    r.remember(format!("I was given {coins} coins."));
    let name = r.name.clone();
    uni.economy.record("treasury", &id, coins, format!("a gift to {name}"));
    app.touch();
    Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
struct TellBody {
    #[serde(flatten)]
    creds: Creds,
    text: String,
}

/// Say something to a resident: they remember it, and think about it soon.
async fn resident_tell(State(app): AppState, headers: HeaderMap, Path(id): Path<String>, Json(b): Json<TellBody>) -> Res {
    let text = clean(&b.text, 400)?;
    let mut uni = app.uni.lock().unwrap();
    let name = match who(&app, &uni, &headers, &b.creds) {
        Ok(Who::Player(_, n)) => n,
        _ => "someone".into(),
    };
    let r = uni.residents.get_mut(&id).ok_or_else(|| err(StatusCode::NOT_FOUND, "no such resident"))?;
    r.remember(format!("{name} told me: \u{201c}{text}\u{201d}"));
    // Think about it within the next few seconds.
    r.last_thought = 0;
    app.touch();
    Ok(Json(json!({ "ok": true })))
}

async fn resident_think(State(app): AppState, headers: HeaderMap, Path(id): Path<String>) -> Res {
    app_only(&app, &headers)?;
    if app.residents_thinking.swap(true, std::sync::atomic::Ordering::Relaxed) {
        return Err(err(StatusCode::CONFLICT, "someone is already thinking"));
    }
    let a2 = app.clone();
    let _ = tokio::task::spawn_blocking(move || crate::residents::think_now(&a2, &id)).await;
    app.residents_thinking.store(false, std::sync::atomic::Ordering::Relaxed);
    Ok(Json(json!({ "ok": true })))
}

async fn resident_retire(State(app): AppState, headers: HeaderMap, Path(id): Path<String>) -> Res {
    app_only(&app, &headers)?;
    let mut uni = app.uni.lock().unwrap();
    let r = uni.residents.remove(&id).ok_or_else(|| err(StatusCode::NOT_FOUND, "no such resident"))?;
    uni.economy.treasury += r.coins;
    uni.economy.record(&id, "treasury", r.coins, format!("{} moved away", r.name));
    drop(uni);
    app.note_commit(format!("{} moved away", r.name));
    app.touch();
    Ok(Json(json!({ "ok": true })))
}

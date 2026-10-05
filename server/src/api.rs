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

async fn world(State(app): AppState, Path(w): Path<String>) -> Res {
    let uni = app.uni.lock().unwrap();
    let world = uni.worlds.get(&w).ok_or_else(|| err(StatusCode::NOT_FOUND, "no such world"))?;
    Ok(Json(json!({ "manifest": world.manifest })))
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
    let mut cells = Vec::new();
    for x in x0..=x1 {
        for z in z0..=z1 {
            if let Some(r) = uni.room(&w, x, z) {
                cells.push(room_summary(&r));
            }
        }
    }
    Ok(Json(json!({ "cells": cells })))
}

async fn room(State(app): AppState, Path((w, x, z)): Path<(String, i32, i32)>) -> Res {
    let uni = app.uni.lock().unwrap();
    let r = uni.room(&w, x, z).ok_or_else(|| err(StatusCode::NOT_FOUND, "no such world"))?;
    Ok(Json(room_view(&app, &uni, &w, &r)))
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

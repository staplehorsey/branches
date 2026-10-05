//! Live layer: one websocket per player per world. Presence, chat, dwell
//! accounting (the architect's main signal) and direct notifications.

use crate::architect;
use crate::model::*;
use crate::state::{App, Msg, Presence};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::response::Response;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

static CONN: AtomicU64 = AtomicU64::new(1);

#[derive(Deserialize)]
#[serde(tag = "t", rename_all = "snake_case")]
enum In {
    Hello { player: String, secret: String, name: String, color: String },
    Move { p: [f32; 3], ry: f32, room: Option<String>, ch: Option<u32> },
    Chat { text: String },
    Admire { room: String, ch: u32 },
}

pub async fn upgrade(ws: WebSocketUpgrade, State(app): State<Arc<App>>, Path(world): Path<String>) -> Response {
    ws.on_upgrade(move |socket| session(socket, app, world))
}

fn clean_name(s: &str) -> String {
    let n: String = s.trim().chars().filter(|c| !c.is_control()).take(24).collect();
    if n.is_empty() { "wanderer".into() } else { n }
}

fn clean_color(s: &str) -> String {
    let ok = s.len() == 7 && s.starts_with('#') && s[1..].chars().all(|c| c.is_ascii_hexdigit());
    if ok { s.to_string() } else { "#ffffff".into() }
}

async fn session(socket: WebSocket, app: Arc<App>, world: String) {
    if !app.uni.lock().unwrap().worlds.contains_key(&world) {
        return;
    }
    let (mut sink, mut stream) = socket.split();

    // The first message must introduce the player.
    let hello = tokio::time::timeout(Duration::from_secs(10), stream.next()).await;
    let Ok(Some(Ok(Message::Text(text)))) = hello else { return };
    let Ok(In::Hello { player, secret, name, color }) = serde_json::from_str::<In>(&text) else { return };
    if player.len() < 8 || player.len() > 64 || secret.len() < 16 {
        return;
    }
    let (name, color) = (clean_name(&name), clean_color(&color));

    // Trust on first use: the first secret seen for an id owns it.
    let unread = {
        let mut uni = app.uni.lock().unwrap();
        let p = uni.players.entry(player.clone()).or_insert_with(|| Player {
            id: player.clone(),
            secret: secret.clone(),
            name: name.clone(),
            color: color.clone(),
            prefs: Default::default(),
            inbox: vec![],
            created: now_ms(),
        });
        if p.secret != secret {
            None
        } else {
            p.name = name.clone();
            p.color = color.clone();
            app.touch();
            Some(p.inbox.iter().filter(|n| !n.read).count())
        }
    };
    let Some(unread) = unread else {
        let _ = sink.send(Message::Text(json!({ "t": "error", "error": "that player id belongs to someone else" }).to_string().into())).await;
        return;
    };

    let conn = CONN.fetch_add(1, Ordering::Relaxed);
    let peers: Vec<_> = {
        let mut pres = app.presence.lock().unwrap();
        let here = pres.entry(world.clone()).or_default();
        let peers = here.values().filter(|p| p.id != player).map(peer_json).collect();
        here.insert(player.clone(), Presence { id: player.clone(), name: name.clone(), color: color.clone(), p: [0.0; 3], ry: 0.0, room: None, ch: None, last_move: Instant::now(), conn });
        peers
    };
    let (dtx, mut drx) = mpsc::unbounded_channel::<Msg>();
    app.direct.lock().unwrap().entry(player.clone()).or_default().push(dtx);
    let mut hub = app.hub(&world).subscribe();

    let welcome = json!({ "t": "welcome", "you": player, "peers": peers, "unread": unread });
    if sink.send(Message::Text(welcome.to_string().into())).await.is_err() {
        cleanup(&app, &world, &player, conn);
        return;
    }
    app.broadcast(&world, &json!({ "t": "join", "id": player, "name": name, "color": color }));

    let writer = tokio::spawn(async move {
        loop {
            let msg = tokio::select! {
                m = hub.recv() => match m {
                    Ok(m) => m,
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(_) => break,
                },
                m = drx.recv() => match m { Some(m) => m, None => break },
            };
            if sink.send(Message::Text(msg.to_string().into())).await.is_err() {
                break;
            }
        }
    });

    let mut last_admire = Instant::now() - Duration::from_secs(10);
    while let Some(Ok(msg)) = stream.next().await {
        let Message::Text(text) = msg else { continue };
        let Ok(input) = serde_json::from_str::<In>(&text) else { continue };
        match input {
            In::Move { p, ry, room, ch } => on_move(&app, &world, &player, p, ry, room, ch),
            In::Chat { text } => {
                let text: String = text.trim().chars().filter(|c| !c.is_control()).take(280).collect();
                if !text.is_empty() {
                    app.broadcast(&world, &json!({ "t": "chat", "id": player, "name": name, "color": color, "text": text }));
                }
            }
            In::Admire { room, ch } => {
                if last_admire.elapsed() < Duration::from_millis(1500) {
                    continue;
                }
                last_admire = Instant::now();
                if let Some((x, z)) = parse_addr(&room) {
                    let tags = architect::record_admire(&mut app.uni.lock().unwrap(), &world, x, z, ch, &player);
                    app.touch();
                    app.send_direct(&player, &json!({ "t": "admired", "addr": room, "tags": tags }));
                }
            }
            In::Hello { .. } => {}
        }
    }
    writer.abort();
    cleanup(&app, &world, &player, conn);
}

fn on_move(app: &App, world: &str, player: &str, p: [f32; 3], ry: f32, room: Option<String>, ch: Option<u32>) {
    if !p.iter().all(|v| v.is_finite()) || !ry.is_finite() {
        return;
    }
    let (dt, entered) = {
        let mut pres = app.presence.lock().unwrap();
        let Some(me) = pres.get_mut(world).and_then(|w| w.get_mut(player)) else { return };
        let dt = me.last_move.elapsed().as_secs_f64().min(1.0);
        let entered = room.is_some() && room != me.room;
        me.p = p;
        me.ry = ry;
        me.room = room.clone();
        me.ch = ch;
        me.last_move = Instant::now();
        (dt, entered)
    };
    let Some((x, z)) = room.as_deref().and_then(parse_addr) else { return };
    let mut uni = app.uni.lock().unwrap();
    architect::record_dwell(&mut uni, world, x, z, ch.unwrap_or(0), player, dt);
    if entered {
        let name = uni.players.get(player).map(|p| p.name.clone()).unwrap_or_default();
        let r = uni.room_mut(world, x, z).unwrap();
        let now = now_ms();
        // One visit entry per player per room per hour keeps the log readable.
        if r.last_visit.get(player).is_none_or(|t| now - t > 3_600_000) {
            r.last_visit.insert(player.to_string(), now);
            let entry = LogEntry { id: new_id(), kind: "visit".into(), who: name, player: Some(player.to_string()), text: "stopped by".into(), at: now };
            push_log(&mut r.log, entry.clone());
            drop(uni);
            app.broadcast(world, &json!({ "t": "log", "addr": addr(x, z), "entry": entry }));
        }
    }
    app.touch();
}

fn peer_json(p: &Presence) -> serde_json::Value {
    json!({ "id": p.id, "name": p.name, "color": p.color, "p": p.p, "ry": p.ry, "room": p.room })
}

fn cleanup(app: &App, world: &str, player: &str, conn: u64) {
    let removed = {
        let mut pres = app.presence.lock().unwrap();
        match pres.get_mut(world) {
            Some(w) if w.get(player).is_some_and(|p| p.conn == conn) => w.remove(player).is_some(),
            _ => false,
        }
    };
    app.direct.lock().unwrap().get_mut(player).map(|v| v.retain(|tx| !tx.is_closed()));
    if removed {
        app.broadcast(world, &json!({ "t": "leave", "id": player }));
    }
}

/// Ten times a second, tell everyone in each world where everyone is.
pub async fn presence_loop(app: Arc<App>) {
    let mut tick = tokio::time::interval(Duration::from_millis(100));
    loop {
        tick.tick().await;
        let snapshots: Vec<(String, serde_json::Value)> = {
            let pres = app.presence.lock().unwrap();
            pres.iter()
                .filter(|(_, players)| players.len() > 1)
                .map(|(w, players)| (w.clone(), json!({ "t": "peers", "list": players.values().map(peer_json).collect::<Vec<_>>() })))
                .collect()
        };
        for (w, msg) in snapshots {
            app.broadcast(&w, &msg);
        }
    }
}

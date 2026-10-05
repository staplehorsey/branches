//! branches-server: the reference host for Branches.
//!
//! One process hosts any number of worlds, serves the web client, and runs
//! an architect that grows rooms asynchronously. Hosts federate simply by
//! linking to each other: a portal's target is an address on another host.

mod api;
mod architect;
mod genesis;
mod model;
mod procgen;
mod state;
mod stories;
mod store;
mod ws;

use axum::Router;
use axum::http::HeaderValue;
use axum::routing::get;
use state::{App, Config};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;
use store::Store;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

/// Builds per world between version tags.
const BUILDS_PER_VERSION: u32 = 3;

/// Write changed files; commit builds right away and everything else
/// (visits, visitor books, heat) every couple of minutes; cut version tags.
fn persist(app: &Arc<App>, last_commit: &mut std::time::Instant) {
    let dirty = app.dirty.swap(false, Ordering::Relaxed);
    let notes: Vec<String> = std::mem::take(&mut *app.commit_notes.lock().unwrap());
    if !dirty && notes.is_empty() {
        return;
    }
    // Worlds due a version: evolve their biome first, so the tag carries it.
    let due: Vec<String> = {
        let mut uni = app.uni.lock().unwrap();
        let ids: Vec<String> = uni.worlds.iter().filter(|(_, w)| w.builds_since_tag >= BUILDS_PER_VERSION).map(|(k, _)| k.clone()).collect();
        for id in &ids {
            let w = uni.worlds.get_mut(id).unwrap();
            genesis::evolve(w, procgen::splitmix(w.manifest.seed as u64 ^ model::now_ms()));
            let n = w.versions.len() + 1;
            let tag = format!("worlds/{id}/v0.{n}.0");
            let grown = w.rooms.values().filter(|r| r.growth() > 0).count();
            w.versions.insert(0, model::Version { tag: tag.clone(), at: model::now_ms(), summary: format!("{grown} houses grown") });
            w.builds_since_tag = 0;
        }
        ids
    };
    let saved = {
        let uni = app.uni.lock().unwrap();
        app.store.save(&uni)
    };
    if let Err(e) = saved {
        tracing::error!("save failed: {e}");
        app.dirty.store(true, Ordering::Relaxed);
        return;
    }
    if notes.is_empty() && due.is_empty() && last_commit.elapsed() < Duration::from_secs(120) {
        return;
    }
    let message = if notes.is_empty() { "Visits, visitor books and heat".to_string() } else { notes.join("\n") };
    match app.store.commit(&message) {
        Ok(Some(_)) => *last_commit = std::time::Instant::now(),
        Ok(None) => {}
        Err(e) => tracing::error!("commit failed: {e}"),
    }
    let uni = app.uni.lock().unwrap();
    for id in due {
        let Some(v) = uni.worlds.get(&id).and_then(|w| w.versions.first()) else { continue };
        if let Err(e) = app.store.tag(&v.tag, &v.summary) {
            tracing::error!("tag {} failed: {e}", v.tag);
        } else {
            tracing::info!("tagged {}", v.tag);
        }
    }
}

/// Is a Branches app already answering on this port?
fn already_running(port: u16) -> bool {
    use std::io::{Read, Write};
    let Ok(mut s) = std::net::TcpStream::connect_timeout(&([127, 0, 0, 1], port).into(), Duration::from_millis(400)) else { return false };
    let _ = s.set_read_timeout(Some(Duration::from_millis(800)));
    let _ = s.write_all(b"GET /.well-known/branches.json HTTP/1.0\r\nHost: localhost\r\n\r\n");
    let mut buf = String::new();
    let _ = s.read_to_string(&mut buf);
    buf.contains("branches/0.1")
}

fn open_browser(url: &str) {
    if std::env::var("BRANCHES_NO_OPEN").is_ok() {
        return;
    }
    let cmd = if cfg!(target_os = "macos") { "open" } else { "xdg-open" };
    let _ = std::process::Command::new(cmd).arg(url).spawn();
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();
    let cfg = Config::from_env();
    let port = cfg.port;
    let url = format!("http://localhost:{port}/");
    if cfg.app && already_running(port) {
        open_browser(&url);
        return;
    }
    let store = match Store::open(&cfg.data_dir) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("cannot open worlds at {}: {e}", cfg.data_dir.display());
            std::process::exit(1);
        }
    };
    let mut uni = store.load();
    uni.seed_builtins();
    let client = cfg.client_dir.clone();
    tracing::info!("worlds in {} (a git repository); client from {}", cfg.data_dir.display(), client.display());
    let app = App::new(cfg, uni, store);
    app.touch();
    app.note_commit("Open the worlds".into());

    tokio::spawn(ws::presence_loop(app.clone()));
    {
        let app = app.clone();
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(Duration::from_secs(1));
            loop {
                tick.tick().await;
                architect::tick(&app);
            }
        });
    }
    {
        let app = app.clone();
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(Duration::from_secs(5));
            let mut last = std::time::Instant::now() - Duration::from_secs(600);
            loop {
                tick.tick().await;
                let a = app.clone();
                last = tokio::task::spawn_blocking(move || {
                    let mut l = last;
                    persist(&a, &mut l);
                    l
                })
                .await
                .unwrap_or(last);
            }
        });
    }

    // Every address is a URL: /w/<world>/<x>,<z> serves the client.
    let index = ServeFile::new(client.join("index.html"));
    let router = Router::new()
        .merge(api::router())
        .route("/ws/{world}", get(ws::upgrade))
        .nest_service("/w", index.clone())
        .fallback_service(ServeDir::new(&client).fallback(index))
        .layer(CorsLayer::permissive())
        // Pages on the public web may talk to the app on this computer
        // (Chrome's Private Network Access asks first).
        .layer(axum::middleware::map_response(|mut res: axum::response::Response| async move {
            res.headers_mut().insert("access-control-allow-private-network", HeaderValue::from_static("true"));
            res
        }))
        .with_state(app.clone());

    let bind: [u8; 4] = if app.cfg.app { [127, 0, 0, 1] } else { [0, 0, 0, 0] };
    let listener = match tokio::net::TcpListener::bind(std::net::SocketAddr::from((bind, port))).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("cannot listen on port {port}: {e}");
            std::process::exit(1);
        }
    };
    tracing::info!("branches listening on {url}");
    if app.cfg.app {
        open_browser(&url);
    }
    let shutdown = {
        let app = app.clone();
        async move {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {}
                _ = app.quit.notified() => {}
            }
            app.dirty.store(true, Ordering::Relaxed);
            app.note_commit("Close the worlds".into());
            let mut l = std::time::Instant::now();
            persist(&app, &mut l);
            tracing::info!("saved; goodbye");
        }
    };
    axum::serve(listener, router).with_graceful_shutdown(shutdown).await.expect("serve");
}

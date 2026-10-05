//! branches-server: the reference host for Branches.
//!
//! One process hosts any number of worlds, serves the web client, and runs
//! an architect that grows rooms asynchronously. Hosts federate simply by
//! linking to each other: a portal's target is an address on another host.

mod api;
mod architect;
mod procgen;
mod model;
mod state;
mod ws;

use axum::Router;
use axum::routing::get;
use model::Universe;
use state::{App, Config};
use std::sync::atomic::Ordering;
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

fn load(cfg: &Config) -> Universe {
    let path = cfg.data_dir.join("universe.json");
    let mut uni: Universe = match std::fs::read(&path) {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_else(|e| {
            tracing::error!("could not parse {}: {e}; starting fresh", path.display());
            Universe::default()
        }),
        Err(_) => Universe::default(),
    };
    uni.seed_builtins();
    uni
}

fn save(app: &App) {
    if !app.dirty.swap(false, Ordering::Relaxed) {
        return;
    }
    let bytes = {
        let uni = app.uni.lock().unwrap();
        serde_json::to_vec(&*uni)
    };
    let result = bytes.map_err(std::io::Error::other).and_then(|bytes| {
        std::fs::create_dir_all(&app.cfg.data_dir)?;
        let tmp = app.cfg.data_dir.join("universe.json.tmp");
        std::fs::write(&tmp, bytes)?;
        std::fs::rename(&tmp, app.cfg.data_dir.join("universe.json"))
    });
    if let Err(e) = result {
        tracing::error!("save failed: {e}");
        app.dirty.store(true, Ordering::Relaxed);
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();
    let cfg = Config::from_env();
    let uni = load(&cfg);
    let port = cfg.port;
    let client = cfg.client_dir.clone();
    tracing::info!("serving client from {}", client.display());
    let app = App::new(cfg, uni);

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
            let mut tick = tokio::time::interval(Duration::from_secs(10));
            loop {
                tick.tick().await;
                let app = app.clone();
                let _ = tokio::task::spawn_blocking(move || save(&app)).await;
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
        .with_state(app.clone());

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await.expect("bind");
    tracing::info!("branches listening on http://localhost:{port}");
    let shutdown = {
        let app = app.clone();
        async move {
            let _ = tokio::signal::ctrl_c().await;
            app.dirty.store(true, Ordering::Relaxed);
            save(&app);
            tracing::info!("saved; goodbye");
        }
    };
    axum::serve(listener, router).with_graceful_shutdown(shutdown).await.expect("serve");
}

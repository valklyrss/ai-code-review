mod api;
mod config;
mod db;
mod error;
mod gitx;
mod mail;
mod model;
mod review;
mod scanner;
mod util;

use api::AppState;
use axum::{http::StatusCode, response::IntoResponse, routing::get_service, Router};
use config::AppConfig;
use std::{net::SocketAddr, path::PathBuf};
use tower_http::{services::{ServeDir, ServeFile}, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,tower_http=info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::load()?;
    config.ensure_dirs()?;
    let pool = db::connect(&config.database.url).await?;
    let state = AppState { config: config.clone(), db: pool };
    api::ai_api::seed_from_config(&state).await?;
    api::settings_api::seed_from_config(&state).await?;

    scanner::scheduler::start(state.clone());
    scanner::worker::start(state.clone());

    let static_dir = PathBuf::from("static");
    let spa = get_service(ServeDir::new(&static_dir).fallback(ServeFile::new(static_dir.join("index.html"))))
        .handle_error(|e| async move { (StatusCode::INTERNAL_SERVER_ERROR, format!("static file error: {e}")).into_response() });

    let app = Router::new()
        .merge(api::routes(state))
        .fallback_service(spa)
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    tracing::info!("server listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

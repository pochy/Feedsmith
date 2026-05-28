mod app;
mod config;
mod db;
mod domain;
mod error;
mod routes;
mod services;

use std::net::SocketAddr;

use anyhow::Context;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "feedsmith=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::Config::from_env();
    let state = app::AppState::new(config.clone()).await?;
    let app = app::router(state);
    let addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .context("invalid APP_HOST/APP_PORT")?;

    tracing::info!(%addr, "starting feedsmith");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

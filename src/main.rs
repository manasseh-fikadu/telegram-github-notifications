mod config;
mod github;
mod telegram;
mod webhook;

use axum::{routing::{get, post}, Router};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Settings;
use crate::telegram::TelegramClient;
use crate::webhook::{handle_webhook, health_check};

#[derive(Clone)]
pub struct AppState {
    settings: Settings,
    telegram: TelegramClient,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "gh_telegram_forwarder=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let settings = Settings::load()?;
    let addr: SocketAddr = format!("{}:{}", settings.server.host, settings.server.port).parse()?;
    
    let telegram = TelegramClient::new(settings.telegram.bot_token.clone());
    
    let state = AppState {
        settings,
        telegram,
    };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/webhook/github", post(handle_webhook))
        .with_state(state);

    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

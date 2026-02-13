use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use tracing::{error, info, warn};

use crate::github::GitHubEvent;
use crate::AppState;

type HmacSha256 = Hmac<Sha256>;

pub async fn handle_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let signature = headers
        .get("x-hub-signature-256")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("sha256="));

    let Some(signature) = signature else {
        warn!("Missing or invalid signature header");
        return StatusCode::UNAUTHORIZED;
    };

    if !verify_signature(&state.settings.github.webhook_secret, &body, signature) {
        warn!("Invalid webhook signature");
        return StatusCode::UNAUTHORIZED;
    }

    let event_type = headers
        .get("x-github-event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    info!(event_type = %event_type, "Received webhook");

    let event = match GitHubEvent::parse(event_type, &body) {
        Ok(e) => e,
        Err(e) => {
            error!(error = %e, "Failed to parse event");
            return StatusCode::BAD_REQUEST;
        }
    };

    if let Err(e) = state.telegram.send_event_notification(&state.settings.routing, &event).await {
        error!(error = %e, "Failed to send notification");
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}

fn verify_signature(secret: &str, body: &[u8], signature: &str) -> bool {
    let Ok(expected_sig) = hex::decode(signature) else {
        return false;
    };

    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };

    mac.update(body);
    let result = mac.finalize();
    let actual_sig = result.into_bytes();

    expected_sig.as_slice() == actual_sig.as_slice()
}

pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

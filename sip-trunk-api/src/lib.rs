pub mod state;
mod auth;
mod proxy;
mod routes;
mod signature;

use axum::routing::{get, post};
use axum::Router;
use tower_http::trace::TraceLayer;

use routes::{management, webhook};
pub use state::ApiState;
pub use signature::{verify_telnyx_signature, SignatureError};

/// Webhook-facing router (port WEBHOOK_PORT, public).
/// Validates Telnyx signatures before proxying to sip-trunk-server.
pub fn build_webhook_router(state: ApiState) -> Router {
    Router::new()
        .route("/webhooks/telnyx", post(webhook::receive_telnyx_webhook))
        .with_state(state)
}

/// Management REST router (port MANAGEMENT_PORT, localhost-only).
/// Enforces Bearer token auth before proxying to sip-trunk-server.
pub fn build_mgmt_router(state: ApiState) -> Router {
    Router::new()
        .route("/calls", get(management::list_calls))
        .route("/calls/outbound", post(management::initiate_outbound))
        .route("/calls/{id}/hangup", post(management::hangup_call))
        .route("/calls/{id}/hold", post(management::hold_call))
        .route("/calls/{id}/transfer", post(management::transfer_call))
        .with_state(state)
}

pub async fn run() -> anyhow::Result<()> {
    let server_base_url = std::env::var("INTERNAL_SERVER_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:3000".to_string());
    let telnyx_public_key = std::env::var("TELNYX_PUBLIC_KEY")
        .unwrap_or_else(|_| String::new());
    let api_secret_key = std::env::var("API_SECRET_KEY")
        .map_err(|_| anyhow::anyhow!("API_SECRET_KEY must be set"))?;

    let state = ApiState {
        server_base_url,
        http: reqwest::Client::new(),
        telnyx_public_key,
        api_secret_key,
    };

    let webhook_app = build_webhook_router(state.clone())
        .layer(TraceLayer::new_for_http());
    let mgmt_app = build_mgmt_router(state)
        .layer(TraceLayer::new_for_http());

    let webhook_port: u16 = std::env::var("WEBHOOK_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()?;
    let mgmt_port: u16 = std::env::var("MANAGEMENT_PORT")
        .unwrap_or_else(|_| "8081".to_string())
        .parse()?;

    let webhook_listener =
        tokio::net::TcpListener::bind(("0.0.0.0", webhook_port)).await?;
    let mgmt_listener =
        tokio::net::TcpListener::bind(("127.0.0.1", mgmt_port)).await?;

    tracing::info!("Webhook server on :{webhook_port}");
    tracing::info!("Management server on :{mgmt_port}");

    tokio::try_join!(
        axum::serve(webhook_listener, webhook_app),
        axum::serve(mgmt_listener, mgmt_app),
    )?;

    Ok(())
}

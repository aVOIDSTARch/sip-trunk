pub mod state;
mod routes;

use axum::routing::{get, post};
use axum::Router;
use sip_trunk_core::{new_call_store, TelnyxClient, VoiceTextClient};
use tower_http::trace::TraceLayer;

use routes::{calls, webhooks};
pub use state::AppState;

/// Builds the internal management router without middleware so tests can use it directly.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/webhooks/telnyx", post(webhooks::handle_telnyx_webhook))
        .route("/calls", get(calls::list_active_calls))
        .route("/calls/outbound", post(calls::initiate_outbound))
        .route("/calls/{id}/hangup", post(calls::hangup_call))
        .route("/calls/{id}/hold", post(calls::hold_call))
        .route("/calls/{id}/transfer", post(calls::transfer_call))
        .with_state(state)
}

pub async fn run() -> anyhow::Result<()> {
    let telnyx = TelnyxClient::from_env()?;
    let voicetext = VoiceTextClient::from_env()?;
    let call_store = new_call_store();

    let state = AppState {
        call_store,
        telnyx,
        voicetext,
    };

    let host = std::env::var("INTERNAL_SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port: u16 = std::env::var("INTERNAL_SERVER_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()?;
    let addr = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("sip-trunk-server listening on {addr}");
    axum::serve(listener, build_router(state).layer(TraceLayer::new_for_http())).await?;
    Ok(())
}

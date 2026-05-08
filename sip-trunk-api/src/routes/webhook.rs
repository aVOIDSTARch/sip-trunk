use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;

use crate::proxy;
use crate::signature::verify_telnyx_signature;
use crate::state::ApiState;

pub async fn receive_telnyx_webhook(
    State(state): State<ApiState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let timestamp = match headers
        .get("telnyx-timestamp")
        .and_then(|v| v.to_str().ok())
    {
        Some(t) => t.to_string(),
        None => {
            tracing::warn!("missing telnyx-timestamp header");
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    let signature = match headers
        .get("telnyx-signature-ed25519")
        .and_then(|v| v.to_str().ok())
    {
        Some(s) => s.to_string(),
        None => {
            tracing::warn!("missing telnyx-signature-ed25519 header");
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    if let Err(e) = verify_telnyx_signature(
        &state.telnyx_public_key,
        &timestamp,
        &body,
        &signature,
    ) {
        tracing::warn!("webhook signature verification failed: {e}");
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let url = format!("{}/webhooks/telnyx", state.server_base_url);
    proxy::proxy_post_raw(&state.http, &url, body).await
}

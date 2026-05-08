use axum::extract::{Path, State};
use axum::Json;
use axum::response::IntoResponse;

use crate::auth::BearerAuth;
use crate::proxy;
use crate::state::ApiState;

pub async fn list_calls(
    _auth: BearerAuth,
    State(state): State<ApiState>,
) -> impl IntoResponse {
    let url = format!("{}/calls", state.server_base_url);
    proxy::proxy_get(&state.http, &url).await
}

pub async fn initiate_outbound(
    _auth: BearerAuth,
    State(state): State<ApiState>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let url = format!("{}/calls/outbound", state.server_base_url);
    proxy::proxy_post(&state.http, &url, body).await
}

pub async fn hangup_call(
    _auth: BearerAuth,
    Path(id): Path<String>,
    State(state): State<ApiState>,
) -> impl IntoResponse {
    let url = format!("{}/calls/{}/hangup", state.server_base_url, id);
    proxy::proxy_post(&state.http, &url, serde_json::Value::Null).await
}

pub async fn hold_call(
    _auth: BearerAuth,
    Path(id): Path<String>,
    State(state): State<ApiState>,
) -> impl IntoResponse {
    let url = format!("{}/calls/{}/hold", state.server_base_url, id);
    proxy::proxy_post(&state.http, &url, serde_json::Value::Null).await
}

pub async fn transfer_call(
    _auth: BearerAuth,
    Path(id): Path<String>,
    State(state): State<ApiState>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let url = format!("{}/calls/{}/transfer", state.server_base_url, id);
    proxy::proxy_post(&state.http, &url, body).await
}

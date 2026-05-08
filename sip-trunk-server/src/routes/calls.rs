use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::Utc;
use sip_trunk_core::store::{get_call, insert_call, list_calls, update_call_status};
use sip_trunk_core::{Call, CallDirection, CallStatus, OutboundCallRequest, TransferRequest};
use uuid::Uuid;

use crate::routes::AppError;
use crate::state::AppState;

pub async fn list_active_calls(
    State(state): State<AppState>,
) -> Json<Vec<Call>> {
    Json(list_calls(&state.call_store))
}

pub async fn initiate_outbound(
    State(state): State<AppState>,
    Json(req): Json<OutboundCallRequest>,
) -> Result<Json<Call>, AppError> {
    let call_control_id = state
        .telnyx
        .initiate_outbound_call(&req.from, &req.to, req.webhook_url)
        .await?;

    let call = Call {
        id: Uuid::new_v4(),
        call_control_id: call_control_id.clone(),
        direction: CallDirection::Outbound,
        from: req.from,
        to: req.to,
        status: CallStatus::Initiated,
        started_at: Utc::now(),
        ended_at: None,
    };

    insert_call(&state.call_store, call.clone())?;
    tracing::info!(call_control_id = %call_control_id, "outbound call initiated");
    Ok(Json(call))
}

pub async fn hangup_call(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<StatusCode, AppError> {
    let call = get_call(&state.call_store, &id)?;
    state.telnyx.hangup_call(&call.call_control_id).await?;
    update_call_status(&state.call_store, &id, CallStatus::Completed)?;
    tracing::info!(call_id = %id, "call hung up via management API");
    Ok(StatusCode::NO_CONTENT)
}

pub async fn hold_call(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<StatusCode, AppError> {
    let call = get_call(&state.call_store, &id)?;
    state.telnyx.hold_call(&call.call_control_id).await?;
    update_call_status(&state.call_store, &id, CallStatus::OnHold)?;
    tracing::info!(call_id = %id, "call placed on hold");
    Ok(StatusCode::NO_CONTENT)
}

pub async fn transfer_call(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(req): Json<TransferRequest>,
) -> Result<StatusCode, AppError> {
    let call = get_call(&state.call_store, &id)?;
    state
        .telnyx
        .transfer_call(&call.call_control_id, &req.to)
        .await?;
    update_call_status(&state.call_store, &id, CallStatus::Transferring)?;
    tracing::info!(call_id = %id, to = %req.to, "call transfer initiated");
    Ok(StatusCode::NO_CONTENT)
}

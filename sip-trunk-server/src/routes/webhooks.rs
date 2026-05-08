use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use chrono::Utc;
use sip_trunk_core::store::{get_call_by_control_id, insert_call, update_call_status};
use sip_trunk_core::{
    Call, CallDirection, CallHangupPayload, CallInitiatedPayload, CallStatus, CoreError,
    TelnyxWebhook,
};
use uuid::Uuid;

use crate::routes::AppError;
use crate::state::AppState;

pub async fn handle_telnyx_webhook(
    State(state): State<AppState>,
    body: axum::body::Bytes,
) -> Result<impl IntoResponse, AppError> {
    let webhook: TelnyxWebhook =
        serde_json::from_slice(&body).map_err(CoreError::Serialization)?;

    match webhook.data.event_type.as_str() {
        "call.initiated" => {
            let payload: CallInitiatedPayload =
                serde_json::from_value(webhook.data.payload).map_err(CoreError::Serialization)?;

            let direction = if payload.direction == "inbound" {
                CallDirection::Inbound
            } else {
                CallDirection::Outbound
            };

            let call = Call {
                id: Uuid::new_v4(),
                call_control_id: payload.call_control_id.clone(),
                direction,
                from: payload.from,
                to: payload.to,
                status: CallStatus::Initiated,
                started_at: Utc::now(),
                ended_at: None,
            };

            insert_call(&state.call_store, call)?;
            state.telnyx.answer_call(&payload.call_control_id).await?;
            tracing::info!(call_control_id = %payload.call_control_id, "call initiated and answered");
        }
        "call.answered" => {
            if let Some(cc_id) = webhook
                .data
                .payload
                .get("call_control_id")
                .and_then(|v| v.as_str())
            {
                if let Ok(call) = get_call_by_control_id(&state.call_store, cc_id) {
                    update_call_status(&state.call_store, &call.id, CallStatus::Answered)?;
                    tracing::info!(call_control_id = %cc_id, "call answered");
                }
            }
        }
        "call.hangup" => {
            let payload: CallHangupPayload =
                serde_json::from_value(webhook.data.payload).map_err(CoreError::Serialization)?;

            if let Ok(call) =
                get_call_by_control_id(&state.call_store, &payload.call_control_id)
            {
                update_call_status(&state.call_store, &call.id, CallStatus::Completed)?;
                tracing::info!(
                    call_control_id = %payload.call_control_id,
                    cause = %payload.hangup_cause,
                    "call hung up"
                );
            }
        }
        other => {
            tracing::debug!(event_type = %other, "unhandled Telnyx event");
        }
    }

    Ok(StatusCode::OK)
}

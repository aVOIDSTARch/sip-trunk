use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type CallId = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CallDirection {
    Inbound,
    Outbound,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CallStatus {
    Initiated,
    Ringing,
    Answered,
    OnHold,
    Transferring,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Call {
    pub id: Uuid,
    pub call_control_id: CallId,
    pub direction: CallDirection,
    pub from: String,
    pub to: String,
    pub status: CallStatus,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct TelnyxWebhook {
    pub data: TelnyxWebhookData,
    pub meta: TelnyxWebhookMeta,
}

#[derive(Debug, Deserialize)]
pub struct TelnyxWebhookData {
    pub event_type: String,
    pub id: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct TelnyxWebhookMeta {
    pub attempt: u32,
    pub delivered_to: String,
}

#[derive(Debug, Deserialize)]
pub struct CallInitiatedPayload {
    pub call_control_id: String,
    pub call_session_id: String,
    pub call_leg_id: String,
    pub direction: String,
    pub from: String,
    pub to: String,
}

#[derive(Debug, Deserialize)]
pub struct CallHangupPayload {
    pub call_control_id: String,
    pub hangup_cause: String,
    pub hangup_source: String,
}

#[derive(Debug, Serialize)]
pub struct OutboundCallRequest {
    pub connection_id: String,
    pub from: String,
    pub to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TransferRequest {
    pub to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
}

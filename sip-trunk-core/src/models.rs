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

#[derive(Debug, Serialize, Deserialize)]
pub struct OutboundCallRequest {
    pub connection_id: String,
    pub from: String,
    pub to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransferRequest {
    pub to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_call_direction_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&CallDirection::Inbound).unwrap(),
            "\"inbound\""
        );
        assert_eq!(
            serde_json::to_string(&CallDirection::Outbound).unwrap(),
            "\"outbound\""
        );
    }

    #[test]
    fn test_call_status_on_hold_snake_case() {
        assert_eq!(
            serde_json::to_string(&CallStatus::OnHold).unwrap(),
            "\"on_hold\""
        );
    }

    #[test]
    fn test_call_roundtrip_json() {
        let call = Call {
            id: Uuid::new_v4(),
            call_control_id: "ctrl_123".to_string(),
            direction: CallDirection::Inbound,
            from: "+15551234567".to_string(),
            to: "+15559876543".to_string(),
            status: CallStatus::Answered,
            started_at: Utc::now(),
            ended_at: None,
        };
        let json = serde_json::to_string(&call).unwrap();
        let back: Call = serde_json::from_str(&json).unwrap();
        assert_eq!(back.call_control_id, call.call_control_id);
        assert_eq!(back.direction, CallDirection::Inbound);
        assert_eq!(back.status, CallStatus::Answered);
        assert!(back.ended_at.is_none());
    }

    #[test]
    fn test_telnyx_webhook_deserializes() {
        let json = r#"{
            "data": {
                "event_type": "call.initiated",
                "id": "evt_001",
                "payload": {
                    "call_control_id": "ctrl_abc",
                    "call_session_id": "sess_abc",
                    "call_leg_id": "leg_abc",
                    "direction": "inbound",
                    "from": "+15551234567",
                    "to": "+15559876543"
                }
            },
            "meta": { "attempt": 1, "delivered_to": "https://example.com" }
        }"#;
        let webhook: TelnyxWebhook = serde_json::from_str(json).unwrap();
        assert_eq!(webhook.data.event_type, "call.initiated");
        assert_eq!(webhook.meta.attempt, 1);
        let payload: CallInitiatedPayload =
            serde_json::from_value(webhook.data.payload).unwrap();
        assert_eq!(payload.call_control_id, "ctrl_abc");
        assert_eq!(payload.direction, "inbound");
    }

    #[test]
    fn test_transfer_request_omits_none_from() {
        let req = TransferRequest { to: "+1555".to_string(), from: None };
        let val = serde_json::to_value(&req).unwrap();
        assert!(val.get("from").is_none(), "None 'from' should be omitted");
    }

    #[test]
    fn test_transfer_request_includes_some_from() {
        let req = TransferRequest {
            to: "+1555".to_string(),
            from: Some("+1666".to_string()),
        };
        let val = serde_json::to_value(&req).unwrap();
        assert_eq!(val["from"], "+1666");
    }

    #[test]
    fn test_outbound_call_request_roundtrip() {
        let req = OutboundCallRequest {
            connection_id: "conn_xyz".to_string(),
            from: "+15551111111".to_string(),
            to: "+15552222222".to_string(),
            webhook_url: Some("https://example.com/hook".to_string()),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: OutboundCallRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.connection_id, "conn_xyz");
        assert_eq!(back.webhook_url, Some("https://example.com/hook".to_string()));
    }
}

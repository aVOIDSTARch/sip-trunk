//! Integration tests for sip-trunk-server HTTP endpoints.
//!
//! Uses axum's `tower::ServiceExt::oneshot` to call handlers in-process
//! without binding to a TCP port. The `AppState` is constructed with a
//! `TelnyxClient` pointed at a wiremock server so no real Telnyx calls are made.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use sip_trunk_core::{new_call_store, TelnyxClient, VoiceTextClient};
use sip_trunk_server::{build_router, AppState};
use tower::ServiceExt;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn test_app(telnyx_base_url: &str) -> axum::Router {
    let state = AppState {
        call_store: new_call_store(),
        telnyx: TelnyxClient::new("test_key".to_string(), "conn_test".to_string())
            .with_base_url(telnyx_base_url.to_string()),
        voicetext: VoiceTextClient::new("http://localhost:9999".to_string(), "k".to_string()),
    };
    build_router(state)
}

fn webhook_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/webhooks/telnyx")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap()
}

fn initiated_webhook(call_control_id: &str) -> serde_json::Value {
    serde_json::json!({
        "data": {
            "event_type": "call.initiated",
            "id": "evt_001",
            "payload": {
                "call_control_id": call_control_id,
                "call_session_id": "sess_001",
                "call_leg_id": "leg_001",
                "direction": "inbound",
                "from": "+15551234567",
                "to": "+15559876543"
            }
        },
        "meta": { "attempt": 1, "delivered_to": "https://example.com" }
    })
}

async fn body_json(response: axum::response::Response) -> serde_json::Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

// ---------------------------------------------------------------------------
// GET /calls
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_list_calls_empty_on_startup() {
    let app = test_app("http://localhost:1").await;

    let response = app
        .oneshot(Request::builder().uri("/calls").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let calls: Vec<serde_json::Value> = serde_json::from_slice(
        &response.into_body().collect().await.unwrap().to_bytes(),
    )
    .unwrap();
    assert!(calls.is_empty());
}

#[tokio::test]
async fn test_list_calls_shows_initiated_call() {
    let mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/calls/ctrl_list_test/actions/answer"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"data":{}})))
        .mount(&mock)
        .await;

    let app = test_app(&mock.uri()).await;

    // Trigger a call.initiated webhook to populate the store
    app.clone()
        .oneshot(webhook_request(initiated_webhook("ctrl_list_test")))
        .await
        .unwrap();

    let response = app
        .oneshot(Request::builder().uri("/calls").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let calls: Vec<serde_json::Value> = serde_json::from_slice(
        &response.into_body().collect().await.unwrap().to_bytes(),
    )
    .unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0]["call_control_id"], "ctrl_list_test");
}

// ---------------------------------------------------------------------------
// POST /webhooks/telnyx
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_webhook_call_initiated_answers_and_returns_200() {
    let mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/calls/ctrl_inbound/actions/answer"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"data":{}})))
        .expect(1)
        .mount(&mock)
        .await;

    let app = test_app(&mock.uri()).await;
    let response = app
        .oneshot(webhook_request(initiated_webhook("ctrl_inbound")))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    mock.verify().await;
}

#[tokio::test]
async fn test_webhook_call_answered_updates_status() {
    let mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/calls/ctrl_answered/actions/answer"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"data":{}})))
        .mount(&mock)
        .await;

    let app = test_app(&mock.uri()).await;

    // First initiate the call
    app.clone()
        .oneshot(webhook_request(initiated_webhook("ctrl_answered")))
        .await
        .unwrap();

    // Then fire call.answered
    let answered = serde_json::json!({
        "data": {
            "event_type": "call.answered",
            "id": "evt_002",
            "payload": { "call_control_id": "ctrl_answered" }
        },
        "meta": { "attempt": 1, "delivered_to": "https://example.com" }
    });
    let response = app.clone().oneshot(webhook_request(answered)).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Verify status is "answered"
    let calls_resp = app
        .oneshot(Request::builder().uri("/calls").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let calls: Vec<serde_json::Value> = serde_json::from_slice(
        &calls_resp.into_body().collect().await.unwrap().to_bytes(),
    )
    .unwrap();
    assert_eq!(calls[0]["status"], "answered");
}

#[tokio::test]
async fn test_webhook_call_hangup_marks_completed() {
    let mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/calls/ctrl_hangup/actions/answer"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"data":{}})))
        .mount(&mock)
        .await;

    let app = test_app(&mock.uri()).await;

    app.clone()
        .oneshot(webhook_request(initiated_webhook("ctrl_hangup")))
        .await
        .unwrap();

    let hangup = serde_json::json!({
        "data": {
            "event_type": "call.hangup",
            "id": "evt_003",
            "payload": {
                "call_control_id": "ctrl_hangup",
                "hangup_cause": "normal_clearing",
                "hangup_source": "remote"
            }
        },
        "meta": { "attempt": 1, "delivered_to": "https://example.com" }
    });
    let response = app.clone().oneshot(webhook_request(hangup)).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let calls: Vec<serde_json::Value> = body_json(
        app.oneshot(Request::builder().uri("/calls").body(Body::empty()).unwrap())
            .await
            .unwrap(),
    )
    .await
    .as_array()
    .unwrap()
    .clone();
    assert_eq!(calls[0]["status"], "completed");
}

#[tokio::test]
async fn test_webhook_unknown_event_returns_200() {
    let app = test_app("http://localhost:1").await;
    let unknown = serde_json::json!({
        "data": { "event_type": "call.bridged", "id": "evt_004", "payload": {} },
        "meta": { "attempt": 1, "delivered_to": "https://example.com" }
    });
    let response = app.oneshot(webhook_request(unknown)).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_webhook_malformed_body_returns_error() {
    let app = test_app("http://localhost:1").await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/webhooks/telnyx")
                .header("content-type", "application/json")
                .body(Body::from(b"not valid json".as_ref()))
                .unwrap(),
        )
        .await
        .unwrap();
    // Malformed JSON should not return 200
    assert_ne!(response.status(), StatusCode::OK);
}

// ---------------------------------------------------------------------------
// POST /calls/{id}/hangup
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_hangup_unknown_call_returns_404() {
    let app = test_app("http://localhost:1").await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/calls/00000000-0000-0000-0000-000000000000/hangup")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// POST /calls/outbound
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_initiate_outbound_call_returns_created_call() {
    let mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/calls"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": { "call_control_id": "ctrl_outbound_001" }
        })))
        .mount(&mock)
        .await;

    let app = test_app(&mock.uri()).await;
    let body = serde_json::json!({
        "connection_id": "conn_test",
        "from": "+15551111111",
        "to": "+15552222222"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/calls/outbound")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let call: serde_json::Value = body_json(response).await;
    assert_eq!(call["call_control_id"], "ctrl_outbound_001");
    assert_eq!(call["direction"], "outbound");
}

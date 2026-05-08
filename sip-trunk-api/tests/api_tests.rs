//! Integration tests for sip-trunk-api gateway endpoints.
//!
//! Two areas tested:
//! 1. Webhook endpoint — Ed25519 signature validation, proxying to sip-trunk-server mock
//! 2. Management endpoint — Bearer auth enforcement, proxying

use axum::body::Body;
use axum::http::{Request, StatusCode};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use ed25519_dalek::{Signer, SigningKey};
use http_body_util::BodyExt;
use sip_trunk_api::{build_mgmt_router, build_webhook_router, ApiState};
use tower::ServiceExt;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ---------------------------------------------------------------------------
// Test fixtures
// ---------------------------------------------------------------------------

const TEST_SEED: [u8; 32] = [0x42u8; 32];

fn test_signing_key() -> SigningKey {
    SigningKey::from_bytes(&TEST_SEED)
}

fn test_pub_key_b64() -> String {
    B64.encode(test_signing_key().verifying_key().as_bytes())
}

fn make_signature(timestamp: &str, body: &[u8]) -> String {
    let mut payload = format!("{timestamp}|").into_bytes();
    payload.extend_from_slice(body);
    B64.encode(test_signing_key().sign(&payload).to_bytes())
}

async fn webhook_state(server_url: &str) -> ApiState {
    ApiState {
        server_base_url: server_url.to_string(),
        http: reqwest::Client::new(),
        telnyx_public_key: test_pub_key_b64(),
        api_secret_key: "test_secret".to_string(),
    }
}

async fn mgmt_state(server_url: &str) -> ApiState {
    webhook_state(server_url).await
}

async fn body_bytes(resp: axum::response::Response) -> Vec<u8> {
    resp.into_body().collect().await.unwrap().to_bytes().to_vec()
}

// ---------------------------------------------------------------------------
// Webhook endpoint — signature verification
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_webhook_missing_timestamp_header_returns_400() {
    let state = webhook_state("http://localhost:1").await;
    let app = build_webhook_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/webhooks/telnyx")
                .header("telnyx-signature-ed25519", "some_sig")
                // telnyx-timestamp intentionally omitted
                .body(Body::from(b"{}".as_ref()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_webhook_missing_signature_header_returns_400() {
    let state = webhook_state("http://localhost:1").await;
    let app = build_webhook_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/webhooks/telnyx")
                .header("telnyx-timestamp", "1700000000")
                // telnyx-signature-ed25519 intentionally omitted
                .body(Body::from(b"{}".as_ref()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_webhook_invalid_signature_returns_401() {
    let state = webhook_state("http://localhost:1").await;
    let app = build_webhook_router(state);
    let body = b"{\"data\":{\"event_type\":\"call.initiated\"}}";

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/webhooks/telnyx")
                .header("telnyx-timestamp", "1700000000")
                .header("telnyx-signature-ed25519", B64.encode([0u8; 64]))
                .header("content-type", "application/json")
                .body(Body::from(body.as_ref()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_webhook_valid_signature_proxies_to_server() {
    let mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/webhooks/telnyx"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock)
        .await;

    let state = webhook_state(&mock.uri()).await;
    let app = build_webhook_router(state);

    let body = b"{\"data\":{\"event_type\":\"call.initiated\"},\"meta\":{\"attempt\":1,\"delivered_to\":\"x\"}}";
    let timestamp = "1700000000";
    let sig = make_signature(timestamp, body);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/webhooks/telnyx")
                .header("telnyx-timestamp", timestamp)
                .header("telnyx-signature-ed25519", sig)
                .header("content-type", "application/json")
                .body(Body::from(body.as_ref()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    mock.verify().await;
}

#[tokio::test]
async fn test_webhook_tampered_body_returns_401() {
    let state = webhook_state("http://localhost:1").await;
    let app = build_webhook_router(state);

    let original = b"{\"data\":{\"event_type\":\"call.initiated\"}}";
    let timestamp = "1700000000";
    let sig = make_signature(timestamp, original);

    // Send a different body with the same signature
    let tampered = b"{\"data\":{\"event_type\":\"call.hangup\"}}";

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/webhooks/telnyx")
                .header("telnyx-timestamp", timestamp)
                .header("telnyx-signature-ed25519", sig)
                .header("content-type", "application/json")
                .body(Body::from(tampered.as_ref()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------------
// Management endpoint — auth enforcement
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_management_no_auth_header_returns_401() {
    let state = mgmt_state("http://localhost:1").await;
    let app = build_mgmt_router(state);

    let response = app
        .oneshot(Request::builder().uri("/calls").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_management_wrong_token_returns_401() {
    let state = mgmt_state("http://localhost:1").await;
    let app = build_mgmt_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/calls")
                .header("Authorization", "Bearer wrong_secret")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_management_valid_token_proxies_list_calls() {
    let mock = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/calls"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .expect(1)
        .mount(&mock)
        .await;

    let state = mgmt_state(&mock.uri()).await;
    let app = build_mgmt_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/calls")
                .header("Authorization", "Bearer test_secret")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_bytes(response).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.is_array());
    mock.verify().await;
}

#[tokio::test]
async fn test_management_hangup_proxies_to_server() {
    let mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/calls/some-uuid/hangup"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&mock)
        .await;

    let state = mgmt_state(&mock.uri()).await;
    let app = build_mgmt_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/calls/some-uuid/hangup")
                .header("Authorization", "Bearer test_secret")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    mock.verify().await;
}

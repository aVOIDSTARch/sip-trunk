# sip-trunk-api

The public-facing gateway. Runs two listeners:

| Port | Purpose | Auth |
|---|---|---|
| `3301` | Telnyx webhook receiver | Ed25519 signature verification |
| `3302` | Management REST API | Bearer token |

All validated requests are proxied to `sip-trunk-server`.

## Webhook endpoint — `POST /webhooks/telnyx`

Telnyx sends a POST request with two signature headers:

```
telnyx-timestamp: 1700000000
telnyx-signature-ed25519: <base64-encoded 64-byte Ed25519 signature>
```

The signed payload is: `"{telnyx-timestamp}|{raw_body}"` (the timestamp string, a pipe character, then the raw request body bytes).

The handler:
1. Reads `telnyx-timestamp` and `telnyx-signature-ed25519` headers — returns `400` if either is missing
2. Calls `verify_telnyx_signature` with the base64 public key from `TELNYX_PUBLIC_KEY` — returns `401` on failure
3. Proxies the raw body to `sip-trunk-server POST /webhooks/telnyx`

### `signature.rs`

```rust
pub fn verify_telnyx_signature(
    public_key_b64: &str,   // base64-encoded 32-byte Ed25519 public key
    timestamp: &str,        // telnyx-timestamp header value
    body: &[u8],            // raw request body bytes
    signature_b64: &str,    // base64-encoded 64-byte signature
) -> Result<(), SignatureError>
```

Uses `ed25519-dalek` v2. Returns `Ok(())` if the signature is valid, `Err(SignatureError)` otherwise. Uses `verify_strict` which rejects weak-point attacks.

## Management endpoints — `/calls*`

All routes require `Authorization: Bearer <API_SECRET_KEY>`. Returns `401` if the header is missing or the token does not match.

| Method | Path | Proxied to |
|---|---|---|
| `GET` | `/calls` | `GET /calls` |
| `POST` | `/calls/outbound` | `POST /calls/outbound` |
| `POST` | `/calls/{id}/hangup` | `POST /calls/{id}/hangup` |
| `POST` | `/calls/{id}/hold` | `POST /calls/{id}/hold` |
| `POST` | `/calls/{id}/transfer` | `POST /calls/{id}/transfer` |

## Running

```sh
cargo run --bin sip-trunk-api
```

| Variable | Required | Description |
|---|---|---|
| `TELNYX_PUBLIC_KEY` | yes | Base64-encoded Ed25519 public key from Telnyx portal |
| `API_SECRET_KEY` | yes | Bearer token for management endpoints |
| `INTERNAL_SERVER_URL` | no | sip-trunk-server URL (default `http://127.0.0.1:3000`) |
| `WEBHOOK_PORT` | no | Webhook listener port (default `3301`) |
| `MANAGEMENT_PORT` | no | Management listener port (default `3302`) |

## Testability

Exposes a library target:

```rust
pub fn build_webhook_router(state: ApiState) -> Router
pub fn build_mgmt_router(state: ApiState) -> Router
pub async fn run() -> anyhow::Result<()>
```

Integration tests use `tower::ServiceExt::oneshot` and a `wiremock` server as the fake sip-trunk-server:

```sh
cargo test -p sip-trunk-api
```

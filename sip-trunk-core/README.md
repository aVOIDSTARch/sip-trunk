# sip-trunk-core

Shared library used by `sip-trunk-server` and (indirectly) the rest of the workspace. Provides all domain models, the in-memory call store, and the two HTTP clients (Telnyx API and the external voice↔text service).

## Modules

### `models`

All domain types, fully `Serialize`/`Deserialize`:

| Type | Description |
|---|---|
| `Call` | Active call record: ID (UUID), `call_control_id`, direction, from/to numbers, status, timestamps |
| `CallDirection` | `Inbound` \| `Outbound` (serializes as `"inbound"` / `"outbound"`) |
| `CallStatus` | `Initiated` → `Ringing` → `Answered` → `OnHold` / `Transferring` → `Completed` / `Failed` |
| `TelnyxWebhook` | Outer envelope for all Telnyx webhook events |
| `CallInitiatedPayload` | Payload for `call.initiated` events |
| `CallHangupPayload` | Payload for `call.hangup` events (includes `hangup_cause`, `hangup_source`) |
| `OutboundCallRequest` | Request body for `POST /v2/calls` |
| `TransferRequest` | Request body for `POST /v2/calls/{id}/actions/transfer` |

### `store`

Thread-safe in-memory call store: `Arc<Mutex<HashMap<Uuid, Call>>>`.

```rust
let store = new_call_store();
insert_call(&store, call)?;
let call = get_call(&store, &id)?;
let call = get_call_by_control_id(&store, "ctrl_abc")?;
let calls = list_calls(&store);
update_call_status(&store, &id, CallStatus::Answered)?;
remove_call(&store, &id)?;
```

Uses `std::sync::Mutex` (not tokio) — the lock is never held across `.await` points.

### `telnyx_client`

Calls the [Telnyx Call Control API v2](https://developers.telnyx.com/api/call-control).

```rust
let client = TelnyxClient::from_env()?;
// or for tests:
let client = TelnyxClient::new(api_key, connection_id)
    .with_base_url("http://mock-server");

client.answer_call("ctrl_abc").await?;
client.hangup_call("ctrl_abc").await?;
client.hold_call("ctrl_abc").await?;
client.unhold_call("ctrl_abc").await?;
client.transfer_call("ctrl_abc", "+15559876543").await?;
let ctrl_id = client.initiate_outbound_call("+15551111111", "+15552222222", None).await?;
```

All call control methods hit `POST /v2/calls/{call_control_id}/actions/{action}`. Non-2xx responses return `CoreError::TelnyxApi { status, body }`.

Environment variables: `TELNYX_API_KEY`, `TELNYX_CONNECTION_ID`.

### `voicetext_client`

Placeholder client for an external voice↔text HTTP service. Both methods currently return empty placeholders and log a warning — wire up the real endpoints once the external service schema is known.

```rust
let client = VoiceTextClient::from_env()?;
let text: String = client.voice_to_text(&audio_bytes).await?;
let audio: Vec<u8> = client.text_to_voice("Hello, caller.").await?;
```

Environment variables: `VOICE_TEXT_API_URL`, `VOICE_TEXT_API_KEY`.

### `error`

```rust
pub enum CoreError {
    Http(reqwest::Error),
    Serialization(serde_json::Error),
    CallNotFound(String),
    MissingEnvVar(String),
    TelnyxApi { status: u16, body: String },
    VoiceTextApi { status: u16, body: String },
}
```

## Public re-exports (`lib.rs`)

```rust
pub use error::{CoreError, Result};
pub use models::*;
pub use store::{new_call_store, CallStore, /* store functions */};
pub use telnyx_client::TelnyxClient;
pub use voicetext_client::VoiceTextClient;
```

## Testing

Unit tests live alongside each module. Run them with:

```sh
cargo test -p sip-trunk-core
```

Tests cover: model serialization round-trips, store CRUD operations, TelnyxClient success and error paths (via wiremock), and VoiceTextClient stub behavior.

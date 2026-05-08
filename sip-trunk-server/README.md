# sip-trunk-server

The internal business-logic server. Binds to `127.0.0.1` only — never exposed to the internet. All public traffic goes through `sip-trunk-api` first.

## Endpoints

### `POST /webhooks/telnyx`

Receives Telnyx call events forwarded by `sip-trunk-api` (after signature verification). Dispatches on `event_type`:

| Event | Action |
|---|---|
| `call.initiated` | Insert call record, call `telnyx.answer_call()` |
| `call.answered` | Update call status → `Answered` |
| `call.hangup` | Update status → `Completed`, record `ended_at` |
| _(anything else)_ | Log at debug level, return `200 OK` |

### `GET /calls`

Returns a JSON array of all call records currently in the store.

### `POST /calls/outbound`

Initiates an outbound call via Telnyx.

Request body:
```json
{ "from": "+15551111111", "to": "+15552222222" }
```

Response: the newly created call record.

### `POST /calls/{id}/hangup`

Hangs up a call by its store UUID. Returns `404` if not found.

### `POST /calls/{id}/hold`

Places a call on hold. Returns `404` if not found.

### `POST /calls/{id}/transfer`

Transfers a call. Request body: `{ "to": "+15559876543" }`.

## State

`AppState` holds:
- `call_store: CallStore` — `Arc<Mutex<HashMap<Uuid, Call>>>` shared across all requests
- `telnyx: TelnyxClient` — handles all calls to `api.telnyx.com`
- `voicetext: VoiceTextClient` — stub for the external voice↔text service

## Running

```sh
cargo run --bin sip-trunk-server
```

Reads configuration from environment / `.env`:

| Variable | Default | Description |
|---|---|---|
| `INTERNAL_SERVER_HOST` | `127.0.0.1` | Bind address |
| `INTERNAL_SERVER_PORT` | `3000` | Listen port |
| `TELNYX_API_KEY` | required | Telnyx API key |
| `TELNYX_CONNECTION_ID` | required | Telnyx connection ID for outbound calls |

## Testability

`sip-trunk-server` exposes a library target with two public items:

```rust
pub fn build_router(state: AppState) -> Router  // no middleware — for tests
pub async fn run() -> anyhow::Result<()>         // full server with TraceLayer
```

Integration tests use `tower::ServiceExt::oneshot` to call handlers in-process without binding a TCP port, and `wiremock` to intercept outgoing Telnyx API calls:

```sh
cargo test -p sip-trunk-server
```

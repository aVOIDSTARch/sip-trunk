# sip-trunk

A Rust workspace that integrates with the [Telnyx](https://telnyx.com) Call Control API to handle inbound and outbound voice calls. It exposes a public webhook gateway, a management REST API, and an admin CLI.

## Architecture

```
Internet
  │
  │  POST /webhooks/telnyx  (Ed25519 verified)
  │  REST  /calls*          (Bearer token auth)
  ▼
sip-trunk-api  :3301 (webhooks)  :3302 (management)
  │
  │  HTTP proxy
  ▼
sip-trunk-server  127.0.0.1:3000  (localhost only)
  │
  ├─► TelnyxClient   →  api.telnyx.com/v2
  └─► VoiceTextClient →  external voice↔text service
```

`sip-trunk-api` is the only internet-facing binary. It verifies Telnyx's Ed25519 webhook signatures before forwarding to the internal server, and enforces Bearer token auth on management endpoints. `sip-trunk-server` never binds to a public interface.

The root `src/main.rs` orchestrates the entire suite — it spawns both services as tokio tasks and shuts them all down on Ctrl+C.

## Crates

| Crate | Type | Purpose |
|---|---|---|
| `sip-logger` | library | `tracing` setup with optional file rotation |
| `sip-trunk-core` | library | Models, call store, Telnyx client, VoiceText client |
| `sip-trunk-server` | binary + library | Business logic, webhook handler, call management HTTP API |
| `sip-trunk-api` | binary + library | Public gateway: signature verification + Bearer auth proxy |
| `sip-trunk-cli` | binary + library | Admin CLI: list/hangup/hold/transfer/outbound calls |

## Telnyx Setup

### 1. Create a Telnyx account

Sign up at [telnyx.com](https://telnyx.com). A phone number is required to receive inbound calls.

### 2. Get your API key

1. Log in to the [Mission Control Portal](https://portal.telnyx.com)
2. Go to **Auth** → **API Keys**
3. Click **Create API Key**
4. Copy the key — this is your `TELNYX_API_KEY`

### 3. Get your Ed25519 public key

Telnyx signs every webhook with an Ed25519 private key. You provide the matching public key to verify signatures.

1. In Mission Control Portal, go to **Auth** → **Public Keys**
2. If no key is listed, click **Add Public Key** — Telnyx generates the pair and gives you the public key
3. Copy the base64-encoded public key — this is your `TELNYX_PUBLIC_KEY`

> The `telnyx-signature-ed25519` header on every webhook is a base64-encoded Ed25519 signature over the string `"{telnyx-timestamp}|{raw_body}"`.

### 4. Create a Call Control Application (connection)

1. Go to **Voice** → **Call Control** → **Applications**
2. Click **Add new application**
3. Set **Application Name** (e.g. `sip-trunk-dev`)
4. Under **Webhook URL**, enter your public webhook endpoint:
   ```
   https://your-domain.example.com/webhooks/telnyx
   ```
   During local development, use a tunnel tool (e.g. `ngrok http 3301`) and paste the HTTPS URL Telnyx requires.
5. Set **Webhook API Version** to **API v2**
6. Save the application
7. Copy the **Connection ID** — this is your `TELNYX_CONNECTION_ID`

### 5. Assign a phone number

1. Go to **Numbers** → **My Numbers**
2. Click a number (or buy one under **Numbers** → **Search & Buy**)
3. Under **Voice**, set **Connection / App** to the application you created above
4. Save

### 6. Configure `.env`

```sh
cp .env.example .env
```

Fill in:

```env
TELNYX_API_KEY=KEYxxxxxxxxxxxxx
TELNYX_PUBLIC_KEY=<base64 public key from step 3>
TELNYX_CONNECTION_ID=<connection ID from step 4>
API_SECRET_KEY=<choose a strong random secret for the management API>
```

## Running

### Run everything (orchestrated)

```sh
cargo run
```

The root binary starts both `sip-trunk-server` and `sip-trunk-api`. Press Ctrl+C to shut down.

### Run services individually

```sh
cargo run --bin sip-trunk-server
cargo run --bin sip-trunk-api
```

### CLI

```sh
# List active calls
API_SECRET_KEY=mysecret cargo run --bin sip-trunk-cli -- calls list

# Hang up a call
cargo run --bin sip-trunk-cli -- --api-key mysecret calls hangup <call-id>

# Outbound call
cargo run --bin sip-trunk-cli -- --api-key mysecret calls outbound --from +15551111111 +15552222222
```

## Testing

```sh
cargo test --workspace
```

All 64 tests run in-process — no live Telnyx credentials or network connections required.

```sh
cargo clippy --workspace -- -D warnings
cargo fmt --all
```

## Environment Variables

| Variable | Required | Default | Description |
|---|---|---|---|
| `TELNYX_API_KEY` | yes | — | Telnyx API key |
| `TELNYX_PUBLIC_KEY` | yes | — | Base64 Ed25519 public key for webhook verification |
| `TELNYX_CONNECTION_ID` | yes | — | Telnyx Call Control Application connection ID |
| `VOICE_TEXT_API_URL` | no | `http://localhost:3300` | External voice↔text service URL |
| `VOICE_TEXT_API_KEY` | no | — | API key for voice↔text service |
| `INTERNAL_SERVER_HOST` | no | `127.0.0.1` | Bind address for sip-trunk-server |
| `INTERNAL_SERVER_PORT` | no | `3000` | Port for sip-trunk-server |
| `INTERNAL_SERVER_URL` | no | `http://127.0.0.1:3000` | URL sip-trunk-api uses to reach the server |
| `WEBHOOK_PORT` | no | `3301` | Public webhook port |
| `MANAGEMENT_PORT` | no | `3302` | Management API port (localhost only) |
| `API_SECRET_KEY` | yes | — | Bearer token for management API |
| `LOG_LEVEL` | no | `info` | Tracing filter (`trace`, `debug`, `info`, `warn`, `error`) |
| `LOG_FILE` | no | — | Log file path prefix (rotates daily); stdout if unset |
| `MANAGEMENT_API_URL` | no | `http://localhost:3302` | CLI default management API URL |

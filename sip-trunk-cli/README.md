# sip-trunk-cli

Admin command-line tool for managing calls via the `sip-trunk-api` management API.

## Installation

```sh
cargo build --release --bin sip-trunk-cli
# binary at: target/release/sip-trunk-cli
```

## Usage

```
sip-trunk --api-url <URL> --api-key <KEY> <COMMAND>
```

Both `--api-url` and `--api-key` can be set via environment variables:

| Flag | Env var | Default |
|---|---|---|
| `--api-url` | `MANAGEMENT_API_URL` | `http://localhost:8081` |
| `--api-key` | `API_SECRET_KEY` | _(required)_ |

### Commands

```sh
# List all active calls
sip-trunk calls list

# Hang up a call
sip-trunk calls hangup <call-uuid>

# Put a call on hold
sip-trunk calls hold <call-uuid>

# Transfer a call to another number
sip-trunk calls transfer <call-uuid> +15559876543

# Initiate an outbound call
sip-trunk calls outbound --from +15551111111 +15552222222
```

### Examples with env vars

```sh
export API_SECRET_KEY=mysecret
export MANAGEMENT_API_URL=http://localhost:3302

sip-trunk calls list
# []

sip-trunk calls outbound --from +15551111111 +15552222222
# {
#   "call_control_id": "ctrl_...",
#   "direction": "outbound",
#   ...
# }
```

## ManagementClient

The `sip_trunk_cli::ManagementClient` struct is the programmatic interface to the management API. It can be used directly in tests or other Rust code:

```rust
use sip_trunk_cli::ManagementClient;

let client = ManagementClient::new(
    "http://localhost:3302".to_string(),
    "mysecret".to_string(),
);

let calls = client.list_calls().await?;
client.hangup("some-uuid").await?;
client.hold("some-uuid").await?;
client.transfer("some-uuid", "+15559876543").await?;
let call = client.outbound("+15551111111", "+15552222222").await?;
```

All methods set `Authorization: Bearer {api_key}` automatically.

## Testing

```sh
cargo test -p sip-trunk-cli
```

Tests cover argument parsing for every subcommand and all `ManagementClient` HTTP methods against a `wiremock` mock server.

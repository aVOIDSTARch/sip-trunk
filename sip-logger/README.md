# sip-logger

A thin wrapper around [`tracing`](https://docs.rs/tracing) and [`tracing-subscriber`](https://docs.rs/tracing-subscriber) that sets up a process-wide structured logger from a single call. Supports optional daily-rotating log files via [`tracing-appender`](https://docs.rs/tracing-appender).

## Usage

Call `init_logger` once at process startup and keep the returned guard alive for the process lifetime. Dropping the guard flushes and closes any open log file.

```rust
use sip_logger::{init_logger, LogConfig};

fn main() {
    let _guard = init_logger(LogConfig {
        level: "info".to_string(),
        file_path: None,          // log to stdout
    });

    tracing::info!("server started");
    // ...
}
```

Log to a rotating daily file:

```rust
let _guard = init_logger(LogConfig {
    level: "debug".to_string(),
    file_path: Some("/var/log/sip-trunk/server.log".to_string()),
});
```

The `file_path` value is split into a directory and a filename prefix. `tracing-appender` creates files like `server.log.2024-01-15`.

## LogConfig

| Field | Type | Default | Description |
|---|---|---|---|
| `level` | `String` | `"info"` | Tracing filter: `trace`, `debug`, `info`, `warn`, `error` |
| `file_path` | `Option<String>` | `None` | Log file path prefix; `None` writes to stdout |

`LogConfig` implements `Default` — `LogConfig::default()` gives `info` level to stdout.

## LoggerGuard

`init_logger` returns a `LoggerGuard`. It wraps the `WorkerGuard` from `tracing-appender`'s non-blocking writer. The guard must be kept alive (typically stored in `main`) — dropping it flushes buffered log records to the file.

## Notes

- Call `init_logger` **exactly once** per process. It installs a global `tracing` subscriber via `.init()`. Calling it a second time will panic.
- Invalid `level` strings fall back to `INFO` with an `eprintln!` warning rather than panicking.
- All sip-trunk binaries initialize the logger at the top of `main` before any async tasks start.

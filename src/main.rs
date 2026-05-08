use tokio::signal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    let _guard = sip_logger::init_logger(sip_logger::LogConfig {
        level: std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
        file_path: std::env::var("LOG_FILE").ok().filter(|s| !s.is_empty()),
    });

    tracing::info!("sip-trunk starting");

    let server_task = tokio::spawn(sip_trunk_server::run());
    let api_task = tokio::spawn(sip_trunk_api::run());

    tokio::select! {
        res = server_task => {
            match res {
                Ok(Ok(())) => tracing::info!("sip-trunk-server exited"),
                Ok(Err(e)) => tracing::error!("sip-trunk-server error: {e}"),
                Err(e) => tracing::error!("sip-trunk-server panicked: {e}"),
            }
        }
        res = api_task => {
            match res {
                Ok(Ok(())) => tracing::info!("sip-trunk-api exited"),
                Ok(Err(e)) => tracing::error!("sip-trunk-api error: {e}"),
                Err(e) => tracing::error!("sip-trunk-api panicked: {e}"),
            }
        }
        _ = signal::ctrl_c() => {
            tracing::info!("Ctrl+C received, shutting down");
        }
    }

    Ok(())
}

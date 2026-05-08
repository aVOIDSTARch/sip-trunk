#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    let _guard = sip_logger::init_logger(sip_logger::LogConfig {
        level: std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
        file_path: std::env::var("LOG_FILE").ok().filter(|s| !s.is_empty()),
    });
    sip_trunk_api::run().await
}

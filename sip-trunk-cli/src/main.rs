use clap::Parser;
use sip_trunk_cli::{Cli, Commands, ManagementClient};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    let _guard = sip_logger::init_logger(sip_logger::LogConfig::default());
    let cli = Cli::parse();
    let client = ManagementClient::new(cli.api_url, cli.api_key);

    match cli.command {
        Commands::Calls { action } => sip_trunk_cli::commands::calls::handle(client, action).await?,
    }

    Ok(())
}

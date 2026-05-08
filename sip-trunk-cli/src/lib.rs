pub mod client;
pub mod commands;

use clap::{Parser, Subcommand};
pub use client::ManagementClient;
pub use commands::calls::CallsAction;

#[derive(Parser)]
#[command(name = "sip-trunk", about = "Manage SIP trunk calls", version)]
pub struct Cli {
    #[arg(long, env = "MANAGEMENT_API_URL", default_value = "http://localhost:8081")]
    pub api_url: String,

    #[arg(long, env = "API_SECRET_KEY")]
    pub api_key: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Call-related commands
    Calls {
        #[command(subcommand)]
        action: CallsAction,
    },
}

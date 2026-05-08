use crate::ManagementClient;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum CallsAction {
    /// List all active calls
    List,
    /// Hang up a call
    Hangup { id: String },
    /// Place a call on hold
    Hold { id: String },
    /// Transfer a call to another number
    Transfer { id: String, to: String },
    /// Initiate an outbound call
    Outbound {
        #[arg(long)]
        from: String,
        to: String,
    },
}

pub async fn handle(client: ManagementClient, action: CallsAction) -> anyhow::Result<()> {
    match action {
        CallsAction::List => {
            let calls = client.list_calls().await?;
            println!("{}", serde_json::to_string_pretty(&calls)?);
        }
        CallsAction::Hangup { id } => {
            client.hangup(&id).await?;
            println!("Call {id} hung up.");
        }
        CallsAction::Hold { id } => {
            client.hold(&id).await?;
            println!("Call {id} placed on hold.");
        }
        CallsAction::Transfer { id, to } => {
            client.transfer(&id, &to).await?;
            println!("Call {id} transfer initiated to {to}.");
        }
        CallsAction::Outbound { from, to } => {
            let call = client.outbound(&from, &to).await?;
            println!("{}", serde_json::to_string_pretty(&call)?);
        }
    }
    Ok(())
}

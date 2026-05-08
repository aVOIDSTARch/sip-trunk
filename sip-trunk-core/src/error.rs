use crate::models::CallId;

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Call not found: {0}")]
    CallNotFound(CallId),

    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),

    #[error("Telnyx API error {status}: {body}")]
    TelnyxApi { status: u16, body: String },

    #[error("VoiceText API error {status}: {body}")]
    VoiceTextApi { status: u16, body: String },
}

pub type Result<T> = std::result::Result<T, CoreError>;

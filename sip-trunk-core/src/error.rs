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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_not_found_message() {
        let e = CoreError::CallNotFound("ctrl_xyz".to_string());
        assert_eq!(e.to_string(), "Call not found: ctrl_xyz");
    }

    #[test]
    fn test_missing_env_var_message() {
        let e = CoreError::MissingEnvVar("TELNYX_API_KEY".to_string());
        assert_eq!(e.to_string(), "Missing environment variable: TELNYX_API_KEY");
    }

    #[test]
    fn test_telnyx_api_error_message() {
        let e = CoreError::TelnyxApi {
            status: 422,
            body: "unprocessable entity".to_string(),
        };
        assert_eq!(e.to_string(), "Telnyx API error 422: unprocessable entity");
    }

    #[test]
    fn test_voicetext_api_error_message() {
        let e = CoreError::VoiceTextApi {
            status: 503,
            body: "unavailable".to_string(),
        };
        assert_eq!(e.to_string(), "VoiceText API error 503: unavailable");
    }
}

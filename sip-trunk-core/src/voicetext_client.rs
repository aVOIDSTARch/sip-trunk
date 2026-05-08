use crate::error::{CoreError, Result};

#[derive(Clone)]
pub struct VoiceTextClient {
    http: reqwest::Client,
    api_url: String,
    api_key: String,
}

impl VoiceTextClient {
    pub fn from_env() -> Result<Self> {
        let api_url = std::env::var("VOICE_TEXT_API_URL")
            .map_err(|_| CoreError::MissingEnvVar("VOICE_TEXT_API_URL".to_string()))?;
        let api_key = std::env::var("VOICE_TEXT_API_KEY")
            .map_err(|_| CoreError::MissingEnvVar("VOICE_TEXT_API_KEY".to_string()))?;
        Ok(Self::new(api_url, api_key))
    }

    pub fn new(api_url: String, api_key: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_url,
            api_key,
        }
    }

    pub async fn voice_to_text(&self, _audio: &[u8]) -> Result<String> {
        tracing::warn!("VoiceTextClient::voice_to_text is not yet implemented — returning placeholder");
        Ok(String::new())
    }

    pub async fn text_to_voice(&self, _text: &str) -> Result<Vec<u8>> {
        tracing::warn!("VoiceTextClient::text_to_voice is not yet implemented — returning placeholder");
        Ok(vec![])
    }
}

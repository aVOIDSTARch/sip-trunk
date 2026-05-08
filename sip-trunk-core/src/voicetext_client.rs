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

#[cfg(test)]
mod tests {
    use super::*;

    fn stub_client() -> VoiceTextClient {
        VoiceTextClient::new("http://localhost:9999".to_string(), "key".to_string())
    }

    #[tokio::test]
    async fn test_voice_to_text_returns_empty_placeholder() {
        let result = stub_client().voice_to_text(b"fake audio bytes").await.unwrap();
        assert_eq!(result, "");
    }

    #[tokio::test]
    async fn test_text_to_voice_returns_empty_placeholder() {
        let result = stub_client().text_to_voice("hello world").await.unwrap();
        assert!(result.is_empty());
    }
}

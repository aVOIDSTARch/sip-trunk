use crate::error::{CoreError, Result};
use crate::models::{OutboundCallRequest, TransferRequest};

#[derive(Clone)]
pub struct TelnyxClient {
    http: reqwest::Client,
    api_key: String,
    base_url: String,
    pub connection_id: String,
}

impl TelnyxClient {
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("TELNYX_API_KEY")
            .map_err(|_| CoreError::MissingEnvVar("TELNYX_API_KEY".to_string()))?;
        let connection_id = std::env::var("TELNYX_CONNECTION_ID")
            .map_err(|_| CoreError::MissingEnvVar("TELNYX_CONNECTION_ID".to_string()))?;
        Ok(Self::new(api_key, connection_id))
    }

    pub fn new(api_key: String, connection_id: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_key,
            base_url: "https://api.telnyx.com/v2".to_string(),
            connection_id,
        }
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }

    pub async fn answer_call(&self, call_control_id: &str) -> Result<()> {
        self.call_action(call_control_id, "answer", None).await
    }

    pub async fn hangup_call(&self, call_control_id: &str) -> Result<()> {
        self.call_action(call_control_id, "hangup", None).await
    }

    pub async fn hold_call(&self, call_control_id: &str) -> Result<()> {
        self.call_action(call_control_id, "hold", None).await
    }

    pub async fn unhold_call(&self, call_control_id: &str) -> Result<()> {
        self.call_action(call_control_id, "unhold", None).await
    }

    pub async fn transfer_call(&self, call_control_id: &str, to: &str) -> Result<()> {
        let body = TransferRequest {
            to: to.to_string(),
            from: None,
        };
        self.call_action(
            call_control_id,
            "transfer",
            Some(serde_json::to_value(body)?),
        )
        .await
    }

    pub async fn initiate_outbound_call(
        &self,
        from: &str,
        to: &str,
        webhook_url: Option<String>,
    ) -> Result<String> {
        let body = OutboundCallRequest {
            connection_id: self.connection_id.clone(),
            from: from.to_string(),
            to: to.to_string(),
            webhook_url,
        };
        let resp = self
            .http
            .post(format!("{}/calls", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(CoreError::TelnyxApi { status, body });
        }

        let data: serde_json::Value = resp.json().await?;
        let call_control_id = data
            .pointer("/data/call_control_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        Ok(call_control_id)
    }

    async fn call_action(
        &self,
        call_control_id: &str,
        action: &str,
        body: Option<serde_json::Value>,
    ) -> Result<()> {
        let url = format!("{}/calls/{}/actions/{}", self.base_url, call_control_id, action);
        let req = self
            .http
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&body.unwrap_or_else(|| serde_json::json!({})));

        let resp = req.send().await?;
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(CoreError::TelnyxApi { status, body });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn mock_client(base_url: &str) -> TelnyxClient {
        TelnyxClient::new("test_key".to_string(), "conn_123".to_string())
            .with_base_url(base_url.to_string())
    }

    #[tokio::test]
    async fn test_answer_call_success() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/calls/ctrl_abc/actions/answer"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": {}})))
            .mount(&server)
            .await;

        mock_client(&server.uri()).answer_call("ctrl_abc").await.unwrap();
    }

    #[tokio::test]
    async fn test_answer_call_telnyx_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/calls/ctrl_bad/actions/answer"))
            .respond_with(ResponseTemplate::new(422).set_body_string(r#"{"errors":[{"detail":"call not found"}]}"#))
            .mount(&server)
            .await;

        let err = mock_client(&server.uri()).answer_call("ctrl_bad").await.unwrap_err();
        assert!(matches!(err, CoreError::TelnyxApi { status: 422, .. }));
    }

    #[tokio::test]
    async fn test_hangup_call_success() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/calls/ctrl_xyz/actions/hangup"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": {}})))
            .mount(&server)
            .await;

        mock_client(&server.uri()).hangup_call("ctrl_xyz").await.unwrap();
    }

    #[tokio::test]
    async fn test_hold_call_success() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/calls/ctrl_xyz/actions/hold"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": {}})))
            .mount(&server)
            .await;

        mock_client(&server.uri()).hold_call("ctrl_xyz").await.unwrap();
    }

    #[tokio::test]
    async fn test_transfer_call_success() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/calls/ctrl_xyz/actions/transfer"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": {}})))
            .mount(&server)
            .await;

        mock_client(&server.uri())
            .transfer_call("ctrl_xyz", "+15553333333")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_initiate_outbound_returns_control_id() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/calls"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": { "call_control_id": "ctrl_outbound_new" }
            })))
            .mount(&server)
            .await;

        let ctrl_id = mock_client(&server.uri())
            .initiate_outbound_call("+15551111111", "+15552222222", None)
            .await
            .unwrap();
        assert_eq!(ctrl_id, "ctrl_outbound_new");
    }

    #[tokio::test]
    async fn test_initiate_outbound_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/calls"))
            .respond_with(ResponseTemplate::new(402).set_body_string("payment required"))
            .mount(&server)
            .await;

        let err = mock_client(&server.uri())
            .initiate_outbound_call("+1555", "+1666", None)
            .await
            .unwrap_err();
        assert!(matches!(err, CoreError::TelnyxApi { status: 402, .. }));
    }
}

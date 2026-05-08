pub struct ManagementClient {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl ManagementClient {
    pub fn new(base_url: String, api_key: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url,
            api_key,
        }
    }

    fn auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        req.bearer_auth(&self.api_key)
    }

    pub async fn list_calls(&self) -> anyhow::Result<Vec<serde_json::Value>> {
        let resp = self
            .auth(self.http.get(format!("{}/calls", self.base_url)))
            .send()
            .await?
            .error_for_status()?;
        Ok(resp.json().await?)
    }

    pub async fn hangup(&self, id: &str) -> anyhow::Result<()> {
        self.auth(
            self.http
                .post(format!("{}/calls/{}/hangup", self.base_url, id)),
        )
        .send()
        .await?
        .error_for_status()?;
        Ok(())
    }

    pub async fn hold(&self, id: &str) -> anyhow::Result<()> {
        self.auth(
            self.http
                .post(format!("{}/calls/{}/hold", self.base_url, id)),
        )
        .send()
        .await?
        .error_for_status()?;
        Ok(())
    }

    pub async fn transfer(&self, id: &str, to: &str) -> anyhow::Result<()> {
        self.auth(
            self.http
                .post(format!("{}/calls/{}/transfer", self.base_url, id))
                .json(&serde_json::json!({ "to": to })),
        )
        .send()
        .await?
        .error_for_status()?;
        Ok(())
    }

    pub async fn outbound(
        &self,
        from: &str,
        to: &str,
    ) -> anyhow::Result<serde_json::Value> {
        let resp = self
            .auth(
                self.http
                    .post(format!("{}/calls/outbound", self.base_url))
                    .json(&serde_json::json!({ "from": from, "to": to })),
            )
            .send()
            .await?
            .error_for_status()?;
        Ok(resp.json().await?)
    }
}

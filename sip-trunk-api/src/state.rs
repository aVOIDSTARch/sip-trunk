#[derive(Clone)]
pub struct ApiState {
    pub server_base_url: String,
    pub http: reqwest::Client,
    pub telnyx_public_key: String,
    pub api_secret_key: String,
}

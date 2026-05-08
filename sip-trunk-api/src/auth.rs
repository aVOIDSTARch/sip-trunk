use axum::extract::FromRequestParts;
use axum::http::{request::Parts, StatusCode};
use axum::Json;

use crate::state::ApiState;

pub struct BearerAuth;

impl FromRequestParts<ApiState> for BearerAuth {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &ApiState,
    ) -> Result<Self, Self::Rejection> {
        let unauthorized = || {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "unauthorized" })),
            )
        };

        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(unauthorized)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(unauthorized)?;

        if token != state.api_secret_key {
            return Err(unauthorized());
        }

        Ok(BearerAuth)
    }
}

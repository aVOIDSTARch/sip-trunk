pub mod calls;
pub mod webhooks;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use sip_trunk_core::CoreError;

pub struct AppError(pub CoreError);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self.0 {
            CoreError::CallNotFound(_) => (StatusCode::NOT_FOUND, self.0.to_string()),
            CoreError::Http(_) | CoreError::TelnyxApi { .. } => {
                (StatusCode::BAD_GATEWAY, self.0.to_string())
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()),
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

impl From<CoreError> for AppError {
    fn from(e: CoreError) -> Self {
        AppError(e)
    }
}

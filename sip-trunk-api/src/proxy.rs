use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub async fn proxy_get(client: &reqwest::Client, url: &str) -> Response {
    match client.get(url).send().await {
        Ok(resp) => relay_response(resp).await,
        Err(e) => {
            tracing::error!("proxy_get failed for {url}: {e}");
            StatusCode::BAD_GATEWAY.into_response()
        }
    }
}

pub async fn proxy_post(
    client: &reqwest::Client,
    url: &str,
    body: serde_json::Value,
) -> Response {
    match client.post(url).json(&body).send().await {
        Ok(resp) => relay_response(resp).await,
        Err(e) => {
            tracing::error!("proxy_post failed for {url}: {e}");
            StatusCode::BAD_GATEWAY.into_response()
        }
    }
}

pub async fn proxy_post_raw(
    client: &reqwest::Client,
    url: &str,
    body: axum::body::Bytes,
) -> Response {
    match client
        .post(url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(body.to_vec())
        .send()
        .await
    {
        Ok(resp) => relay_response(resp).await,
        Err(e) => {
            tracing::error!("proxy_post_raw failed for {url}: {e}");
            StatusCode::BAD_GATEWAY.into_response()
        }
    }
}

async fn relay_response(resp: reqwest::Response) -> Response {
    let status = axum::http::StatusCode::from_u16(resp.status().as_u16())
        .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    let body = resp.bytes().await.unwrap_or_default();
    (status, body).into_response()
}

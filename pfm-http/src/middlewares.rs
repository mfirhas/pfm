use std::{
    collections::HashMap,
    fs,
    sync::{Arc, LazyLock},
};

use axum::{
    body::Body,
    extract::Request,
    http::{HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::dto::*;

pub(crate) async fn processing_time_middleware(req: Request<Body>, next: Next) -> Response {
    let start = tokio::time::Instant::now();
    let mut response = next.run(req).await;
    let duration_ms = start.elapsed().as_millis();

    let value = format!("pfm-be;dur={}", duration_ms);

    if let Ok(server_timing_value) = HeaderValue::from_str(&value) {
        response
            .headers_mut()
            .insert("Server-Timing", server_timing_value);
    }

    response
}

// contains api keys for client to access these apis
static API_KEYS: LazyLock<Arc<HashMap<String, String>>> = LazyLock::new(|| {
    let content = fs::read_to_string("api_keys.json")
        .expect("Loading api_keys.json: Failed to read api_keys.json");
    let parsed: HashMap<String, String> =
        serde_json::from_str(&content).expect("Loading api_keys.json: Invalid JSON format");
    Arc::new(parsed)
});

pub(crate) async fn api_key_middleware(
    req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let Some(header_val) = req.headers().get("x-api-key").and_then(|v| v.to_str().ok()) else {
        return Err(AppError::Unauthorized(
            "request requires api key set in header in x-api-key".to_string(),
        ));
    };

    if !API_KEYS.values().any(|v| v == header_val) {
        return Err(AppError::Unauthorized(
            "request's api key is invalid".to_string(),
        ));
    }

    Ok(next.run(req).await)
}

const REQUEST_ID_HEADER_NAME: &str = "x-request-id";

pub async fn request_id_middleware(mut req: Request<Body>, next: Next) -> Response {
    // Check for existing request ID or generate one
    let request_id = req
        .headers()
        .get(REQUEST_ID_HEADER_NAME)
        .cloned()
        .unwrap_or_else(|| HeaderValue::from_str(&Uuid::new_v4().to_string()).unwrap());

    // Insert it into request extensions so it's accessible in handlers
    req.extensions_mut().insert(request_id.clone());

    // Continue down the stack
    let mut response = next.run(req).await;

    // Insert it into response headers
    response
        .headers_mut()
        .insert(REQUEST_ID_HEADER_NAME, request_id);

    response
}

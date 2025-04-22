use std::{
    collections::HashMap,
    fs,
    sync::{Arc, LazyLock},
};

use axum::{body::Body, extract::Request, http::HeaderValue, middleware::Next, response::Response};
use tracing::info_span;
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
    let header_api_key = req.headers().get("x-api-key").and_then(|v| v.to_str().ok());

    let api_key_val = if let Some(header_api_key_val) = header_api_key {
        header_api_key_val.to_string()
    } else {
        // check from query param if header not set
        let Some(query_param_api_key_val) = req.uri().query().and_then(|query| {
            url::form_urlencoded::parse(query.as_bytes())
                .find(|(k, _)| k == "api_key")
                .map(|(_, v)| v.to_string())
        }) else {
            return Err(AppError::Unauthorized(
                "request requires api key set in header as x-api-key OR in query param as api_key"
                    .to_string(),
            ));
        };
        query_param_api_key_val
    };

    if !API_KEYS.values().any(|v| v == &api_key_val) {
        return Err(AppError::Unauthorized(
            "request's api key is invalid".to_string(),
        ));
    }

    Ok(next.run(req).await)
}

const REQUEST_ID_HEADER_NAME: &str = "x-request-id";

const CORRELATION_ID_HEADER_NAME: &str = "x-correlation-id";

pub async fn tracing_middleware(mut req: Request, next: Next) -> Response {
    // get request id from header, or generate one
    let request_id_header_val = req
        .headers()
        .get(REQUEST_ID_HEADER_NAME)
        .cloned()
        .unwrap_or_else(|| {
            HeaderValue::from_str(&Uuid::new_v4().to_string())
                .unwrap_or_else(|_| HeaderValue::from_static(""))
        });
    let request_id = request_id_header_val.to_str().ok().unwrap_or_default();

    // generate correlation id for tracing
    let correlation_id = Uuid::new_v4();
    let correlation_id_header_val = HeaderValue::from_str(&correlation_id.to_string())
        .unwrap_or_else(|_| HeaderValue::from_static(""));
    let method = req.method().clone();
    let uri = req.uri().clone();

    let span = info_span!(
        "tracing_middleware",
        %correlation_id,
        %request_id,
        method = %method,
        uri = %uri
    );

    req.extensions_mut().insert(request_id_header_val.clone());
    req.extensions_mut()
        .insert(correlation_id_header_val.clone());

    let _enter = span.enter();

    tracing::info!("--------------------Request received--------------------");

    let mut response = next.run(req).await;

    response
        .headers_mut()
        .insert(REQUEST_ID_HEADER_NAME, request_id_header_val);
    response
        .headers_mut()
        .insert(CORRELATION_ID_HEADER_NAME, correlation_id_header_val);

    response
}

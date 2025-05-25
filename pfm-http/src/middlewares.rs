use std::{
    collections::HashMap,
    fs,
    sync::{Arc, LazyLock},
};

use axum::{body::Body, extract::Request, http::HeaderValue, middleware::Next, response::Response};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{info_span, Instrument};
use uuid::Uuid;

use crate::{dto::*, global};

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

pub(crate) async fn admin_password_middleware(
    req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let admin_pass_header = req
        .headers()
        .get("x-admin-password")
        .and_then(|v| v.to_str().ok());

    if let Some(pass) = admin_pass_header {
        if pass != global::config().admin_password.as_str() {
            return Err(AppError::Unauthorized("admin unauthorized".to_string()));
        }
    } else {
        return Err(AppError::BadRequest(
            "admin password not provided".to_string(),
        ));
    }

    Ok(next.run(req).await)
}

#[derive(Debug, Serialize, Deserialize)]
struct RateLimitData {
    date_time: Option<DateTime<Utc>>,
    /// current count
    count: u32,
    /// the limit in each unit, e.g. 2 per day, 2 is the max
    max: u32,
    /// unit in seconds, limit per unit, e.g. 2 per day, day is the unit in seconds(86400secs)
    unit: u64,
}

type RateLimitMap = HashMap<String, RateLimitData>;

// contains admin api rate limit counts data
static RATE_LIMIT: LazyLock<RwLock<RateLimitMap>> = LazyLock::new(|| {
    let content = fs::read_to_string("admin_rate_limit.json")
        .expect("Loading admin_rate_limit.json: failed reading the file");
    let parsed: RateLimitMap =
        serde_json::from_str(&content).expect("Loading admin_rate_limit.json: Invalid json format");
    tracing::info!("admin rate limit data: {:?}", &parsed);
    RwLock::new(parsed)
});

pub(crate) async fn forex_admin_rate_limit_middleware(
    req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let mut rate_limit_guard = RATE_LIMIT.write().await;
    let rate_limit_data = rate_limit_guard.get_mut("forex_admin");
    if let Some(rate_limit) = rate_limit_data {
        if !check_rate_limit(rate_limit) {
            return Err(AppError::Unauthorized(
                "forex admin rate limit exceeded".to_string(),
            ));
        }
        drop(rate_limit_guard);
        Ok(next.run(req).await)
    } else {
        drop(rate_limit_guard);
        Err(AppError::Unauthorized(
            "forex admin rate limit data not found".to_string(),
        ))
    }
}

fn check_rate_limit(data: &mut RateLimitData) -> bool {
    let now = Utc::now();

    // If this is the first request
    if data.date_time.is_none() {
        data.date_time = Some(now);
        data.count = 1;
        return true;
    }

    let last_time = data.date_time.unwrap();
    let elapsed = (now - last_time).num_seconds() as u64;

    // If the time window has passed, reset the counter
    if elapsed >= data.unit {
        data.date_time = Some(now);
        data.count = 1;
        return true;
    }

    // If we're still within the time window
    if data.count < data.max {
        // Increment the counter and allow the action
        data.count += 1;
        return true;
    }

    // Rate limit hit
    false
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

    // let _span = span.enter();

    tracing::info!("--------------------Request received--------------------");

    let mut response = async move { next.run(req).await }.instrument(span).await;

    response
        .headers_mut()
        .insert(REQUEST_ID_HEADER_NAME, request_id_header_val);
    response
        .headers_mut()
        .insert(CORRELATION_ID_HEADER_NAME, correlation_id_header_val);

    response
}

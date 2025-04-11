use std::{
    collections::{HashMap, HashSet},
    fs,
    marker::PhantomData,
    sync::{Arc, LazyLock},
};

use axum::{
    body::Body,
    http::{HeaderMap, HeaderValue, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use pfm_core::{forex::ForexError, utils::get_config};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tower::ServiceBuilder;

mod handler;

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpResponse<T> {
    #[serde(rename = "data")]
    pub data: Option<T>,

    #[serde(rename = "error")]
    pub error: Option<String>,

    #[serde(skip)]
    _marker: PhantomData<T>,
}

impl<T> HttpResponse<T> {
    pub fn ok(
        data: T,
        headers: Option<HeaderMap>,
    ) -> (StatusCode, Option<HeaderMap>, Json<HttpResponse<T>>) {
        (StatusCode::OK, headers, Json(Self::new(data)))
    }

    fn new(data: T) -> Self {
        Self {
            data: Some(data),
            error: None,
            _marker: PhantomData,
        }
    }

    fn err(error: String) -> Self {
        Self {
            data: None,
            error: Some(error),
            _marker: PhantomData,
        }
    }
}

async fn processing_time_middleware(req: Request<Body>, next: Next) -> Response {
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

async fn api_key_middleware(req: Request<Body>, next: Next) -> Result<Response, AppError> {
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

#[tokio::main]
async fn main() {
    let cfg = get_config::<Config>("HTTP_").expect("failed getting config");

    let core_cfg = pfm_core::global::config();
    let http_client = pfm_core::global::http_client();
    let storage_fs = pfm_core::global::storage_fs();
    let forex = pfm_core::forex_impl::open_exchange_api::Api::new(
        &core_cfg.forex_open_exchange_api_key,
        http_client,
    );
    let storage = pfm_core::forex_impl::forex_storage::ForexStorageImpl::new(storage_fs);

    let app_ctx = AppContext {
        forex_rates: forex.clone(),
        forex_historical_rates: forex.clone(),
        forex_storage: storage.clone(),
    };

    let root = Router::new().route("/ping", get(ping));

    let forex_routes = Router::new()
        .route("/convert", get(handler::convert_handler))
        .route("/rates", get(handler::get_rates_handler))
        .route("/timeseries", get(handler::get_timeseries_handler))
        .layer(ServiceBuilder::new().layer(axum::middleware::from_fn(api_key_middleware)));

    let routes_group = Router::new()
        .nest("/", root)
        .nest("/forex", forex_routes)
        .with_state(app_ctx)
        .layer(ServiceBuilder::new().layer(axum::middleware::from_fn(processing_time_middleware)));

    let addr = ("127.0.0.1", cfg.http_port);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("httpserver: failed listening to tcp");

    println!(
        "server is listening on {}",
        listener.local_addr().expect("httpserver: invalid address")
    );

    axum::serve(listener, routes_group)
        .await
        .expect("httpserver failed");
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Config {
    #[serde(alias = "HTTP_PORT")]
    pub http_port: u16,
}

#[derive(Clone)]
pub(crate) struct AppContext<FX, FHX, FS> {
    forex_rates: FX,
    forex_historical_rates: FHX,
    forex_storage: FS,
}

async fn ping() -> impl IntoResponse {
    "pong"
}

#[derive(Debug, Error, Serialize)]
pub enum AppError {
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Invalid input: {0}")]
    BadRequest(String),

    #[error("Internal error: {0}")]
    InternalServerError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status_code, err_msg) = match self {
            Self::Unauthorized(err) => (StatusCode::UNAUTHORIZED, err),
            Self::BadRequest(err) => (StatusCode::BAD_REQUEST, err),
            Self::InternalServerError(err) => (StatusCode::INTERNAL_SERVER_ERROR, err),
        };

        let resp = HttpResponse::<((), ())>::err(err_msg);

        (status_code, Json(resp)).into_response()
    }
}

impl From<ForexError> for AppError {
    fn from(value: ForexError) -> Self {
        match value {
            ForexError::ClientError(v) => Self::BadRequest(v.to_string()),
            ForexError::InternalError(v) => Self::InternalServerError(v.to_string()),
        }
    }
}

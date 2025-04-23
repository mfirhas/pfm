use std::{fmt, marker::PhantomData};

use async_trait::async_trait;
use axum::{
    extract::{FromRequestParts, Query},
    http::{request::Parts, HeaderValue},
};
use axum::{
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use pfm_core::forex::ForexError;
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpResponse<T> {
    #[serde(rename = "data", skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,

    #[serde(rename = "error", skip_serializing_if = "Option::is_none")]
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

#[derive(Debug, Error, Serialize)]
pub enum AppError {
    #[error("Not found: {0}")]
    NoContent(String),

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
            Self::NoContent(err) => (StatusCode::NO_CONTENT, err),
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
        tracing::error!("ForexError: {}", value);
        match value {
            ForexError::Error(v) => Self::NoContent(v.to_string()),
            ForexError::ClientError(v) => Self::BadRequest(v.to_string()),
            ForexError::InternalError(v) => Self::InternalServerError(v.to_string()),
        }
    }
}

/// trait to give error massage to inputs(query params, path params,or request body)
pub trait BadRequestErrMsg {
    fn bad_request_err_msg() -> &'static str {
        "Missing or invalid input"
    }
}

pub trait Validate {
    fn validate(&self) -> Result<(), AppError>;
}

/// custom query to handle if query params are missing.
pub struct CustomQuery<T>(pub T);

#[async_trait]
impl<S, T> FromRequestParts<S> for CustomQuery<T>
where
    T: Validate + BadRequestErrMsg + DeserializeOwned + Send + Sync,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let Query(params) = Query::<T>::from_request_parts(parts, _state)
            .await
            .map_err(|_| AppError::BadRequest(T::bad_request_err_msg().to_string()))?;

        params.validate()?;

        Ok(CustomQuery(params))
    }
}

#[derive(Clone)]
pub struct RequestId(pub Uuid);

impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for RequestId
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Try to get from header
        if let Some(val) = parts.headers.get("x-request-id") {
            let request_id = val
                .to_str()
                .map_err(|err| AppError::BadRequest(err.to_string()))?
                .parse::<Uuid>()
                .map_err(|err| AppError::BadRequest(err.to_string()))?;

            return Ok(RequestId(request_id));
        }

        // Fallback to extensions
        if let Some(val) = parts.extensions.get::<HeaderValue>() {
            let request_id = val
                .to_str()
                .map_err(|err| AppError::BadRequest(err.to_string()))?
                .parse::<Uuid>()
                .map_err(|err| AppError::BadRequest(err.to_string()))?;

            return Ok(RequestId(request_id));
        }

        Err(AppError::BadRequest(
            "request doesn't contain request id".to_string(),
        ))
    }
}

// deserialize date from YYYY-MM-DD into YYYY-MM-DDThh:mm:ssZ utc
pub fn deserialize_optional_date<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(value) => {
            if let Ok(date) = NaiveDate::parse_from_str(&value, "%Y-%m-%d") {
                let dt = Utc.from_utc_datetime(
                    &date
                        .and_hms_opt(0, 0, 0)
                        .ok_or_else(|| serde::de::Error::custom("Invalid time conversion"))?,
                );
                return Ok(Some(dt));
            }
            Err(serde::de::Error::custom(
                "Invalid date format, expected YYYY-MM-DD",
            ))
        }
        None => Ok(None),
    }
}

pub fn deserialize_date<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    let date = NaiveDate::parse_from_str(&s, "%Y-%m-%d")
        .map_err(|_| serde::de::Error::custom("Invalid date format, expected YYYY-MM-DD"))?;

    let dt = Utc.from_utc_datetime(
        &date
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| serde::de::Error::custom("Invalid time conversion"))?,
    );

    Ok(dt)
}

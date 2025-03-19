use std::{error::Error, marker::PhantomData, str::FromStr};

use anyhow::anyhow;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use pfm_core::{
    forex::{
        currency::Currency, entity::ConversionResponse, entity::HttpResponse, entity::Order,
        interface::ForexError, interface::ForexHistoricalRates, interface::ForexRates,
        interface::ForexStorage, Money,
    },
    forex_impl::forex_storage::ForexStorageImpl,
    forex_impl::open_exchange_api::Api,
    utils::get_config,
};
use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

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
        .route(
            "/convert",
            get(convert_handler::<Api, Api, ForexStorageImpl>),
        )
        .route("/batch-convert", get(batch_convert_handler))
        .route("/latest", get(get_latest_rates_handler))
        .route("/historical", get(get_historical_rates_handler))
        .route("/latest-list", get(get_latest_list_handler))
        .route("/historical-list", get(get_historical_list_handler));

    let routes_group = Router::new()
        .nest("/", root)
        .nest("/forex", forex_routes)
        .with_state(app_ctx);

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
enum AppError {
    #[error("Invalid input: {0}")]
    BadRequest(String),

    #[error("Internal error: {0}")]
    InternalServerError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status_code, err_msg) = match self {
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

#[derive(Debug, Deserialize, Serialize)]
pub struct ConvertQuery {
    #[serde(rename = "from")]
    pub from: String,

    #[serde(rename = "to")]
    pub to: String,
}

async fn convert_handler<FX, FHX, FS>(
    State(ctx): State<AppContext<FX, FHX, FS>>,
    Query(params): Query<ConvertQuery>,
) -> Result<impl IntoResponse, AppError>
where
    FX: ForexRates,
    FHX: ForexHistoricalRates,
    FS: ForexStorage,
{
    let money = Money::from_str(&params.from)?;
    let currency = Currency::from_str(&params.to)?;
    let ret = pfm_core::forex::service::convert(&ctx.forex_storage, money, currency)
        .await
        .map(|ret| HttpResponse::new(ret))?;

    Ok((StatusCode::OK, Json(ret)))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BatchConvertQuery {
    /// separated by `;`, e.g. ?from=USD 1;USD 1,000;
    #[serde(deserialize_with = "from_seq")]
    pub from: Vec<String>,

    pub to: Currency,
}

fn from_seq<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = <String>::deserialize(deserializer)?;

    s.split(';')
        .map(|i| String::from_str(i))
        .collect::<Result<Vec<_>, _>>()
        .map_err(serde::de::Error::custom)
}

async fn batch_convert_handler<FX, FHX, FS>(
    State(ctx): State<AppContext<FX, FHX, FS>>,
    Query(params): Query<BatchConvertQuery>,
) -> Result<impl IntoResponse, AppError>
where
    FX: ForexRates,
    FHX: ForexHistoricalRates,
    FS: ForexStorage,
{
    let input = {
        let mut vecs: Vec<Money> = vec![];
        for x in params.from {
            let money = Money::from_str(&x)?;
            vecs.push(money);
        }
        vecs
    };

    let ret = pfm_core::forex::service::batch_convert(&ctx.forex_storage, input, params.to)
        .await
        .map(|ret| HttpResponse::new(ret))?;

    Ok((StatusCode::OK, Json(ret)))
}

async fn get_latest_rates_handler(
    State(ctx): State<AppContext<impl ForexRates, impl ForexHistoricalRates, impl ForexStorage>>,
) -> Result<impl IntoResponse, AppError> {
    Ok((
        StatusCode::OK,
        Json(
            ctx.forex_storage
                .get_latest()
                .await
                .map(|ret| HttpResponse::new(ret))?,
        ),
    ))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct GetHistoricalRatesQuery {
    date: String,
}

async fn get_historical_rates_handler(
    State(ctx): State<AppContext<impl ForexRates, impl ForexHistoricalRates, impl ForexStorage>>,
    Query(query): Query<GetHistoricalRatesQuery>,
) -> Result<impl IntoResponse, AppError> {
    let date = NaiveDate::parse_from_str(&query.date, "%Y-%m-%d").unwrap();
    let date = NaiveDateTime::new(date, NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    let date = Utc.from_utc_datetime(&date);

    Ok((
        StatusCode::OK,
        Json(
            ctx.forex_storage
                .get_historical(date)
                .await
                .map(|ret| HttpResponse::new(ret))?,
        ),
    ))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct GetListQuery {
    page: u32,
    size: u32,
    order: Order,
}

async fn get_latest_list_handler(
    State(ctx): State<AppContext<impl ForexRates, impl ForexHistoricalRates, impl ForexStorage>>,
    Query(query): Query<GetListQuery>,
) -> Result<impl IntoResponse, AppError> {
    Ok((
        StatusCode::OK,
        Json(
            ctx.forex_storage
                .get_latest_list(query.page, query.size, query.order)
                .await
                .map(|ret| HttpResponse::new(ret))?,
        ),
    ))
}

async fn get_historical_list_handler(
    State(ctx): State<AppContext<impl ForexRates, impl ForexHistoricalRates, impl ForexStorage>>,
    Query(query): Query<GetListQuery>,
) -> Result<impl IntoResponse, AppError> {
    Ok((
        StatusCode::OK,
        Json(
            ctx.forex_storage
                .get_historical_list(query.page, query.size, query.order)
                .await
                .map(|ret| HttpResponse::new(ret))?,
        ),
    ))
}

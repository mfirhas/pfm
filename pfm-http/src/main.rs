use std::{marker::PhantomData, str::FromStr};

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
        ConversionResponse, Currencies, ForexHistoricalRates, ForexRates, ForexStorage, Money,
    },
    forex_impl::open_exchange_api::Api,
    forex_storage_impl::forex_storage::ForexStorageImpl,
    utils::get_config,
};
use serde::{Deserialize, Serialize};

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
    let storage = pfm_core::forex_storage_impl::forex_storage::ForexStorageImpl::new(storage_fs);

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
        .route("/latest", get(get_latest_rates_handler))
        .route("/historical", get(get_historical_rates_handler));

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

#[derive(Debug, Serialize, Deserialize)]
pub struct Response<T> {
    #[serde(rename = "data")]
    pub data: Option<T>,

    #[serde(rename = "error")]
    pub error: Option<String>,

    #[serde(skip)]
    _marker: PhantomData<T>,
}

impl<T> Response<T> {
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
) -> impl IntoResponse
where
    FX: ForexRates,
    FHX: ForexHistoricalRates,
    FS: ForexStorage,
{
    let ret = match (
        Money::from_str(&params.from),
        Currencies::from_str(&params.to),
    ) {
        (Ok(money), Ok(curr)) => {
            let conversion_ret = pfm_core::forex::convert(&ctx.forex_storage, money, curr)
                .await
                .map(|ret| Response::new(ret))
                .map_err(|err| Response::<ConversionResponse>::err(err.to_string()))
                .unwrap_or(Response::err("failed to convert".to_string()));
            conversion_ret
        }
        (Err(err), _) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(Response::err(
                    anyhow!("bad request: from money: {}", err).to_string(),
                )),
            )
        }
        (_, Err(err)) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(Response::err(
                    anyhow!("bad request: target currency: {}", err).to_string(),
                )),
            )
        }
    };

    (StatusCode::OK, Json(ret))
}

async fn get_latest_rates_handler(
    State(ctx): State<AppContext<impl ForexRates, impl ForexHistoricalRates, impl ForexStorage>>,
) -> impl IntoResponse {
    match ctx.forex_storage.get_latest().await {
        Ok(resp) => (StatusCode::OK, Json(Response::new(resp))),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(Response::err(err.to_string())),
        ),
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct GetHistoricalRatesQuery {
    date: String,
}

async fn get_historical_rates_handler(
    State(ctx): State<AppContext<impl ForexRates, impl ForexHistoricalRates, impl ForexStorage>>,
    Query(query): Query<GetHistoricalRatesQuery>,
) -> impl IntoResponse {
    let date = NaiveDate::parse_from_str(&query.date, "%Y-%m-%d").unwrap();
    let date = NaiveDateTime::new(date, NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    let date = Utc.from_utc_datetime(&date);
    match ctx.forex_storage.get_historical(date).await {
        Ok(ret) => (StatusCode::OK, Json(Response::new(ret))),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(Response::err(err.to_string())),
        ),
    }
}

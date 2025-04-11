use std::str::FromStr;

use axum::{
    async_trait,
    extract::{FromRequestParts, Query, State},
    http::{request::Parts, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
use pfm_core::forex::{
    entity::{HistoricalRates, Rates, RatesData, RatesResponse},
    interface::{ForexHistoricalRates, ForexRates, ForexStorage},
    Currency, Money,
};
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize};

use crate::{AppContext, AppError, HttpResponse};
use pfm_core::forex::service;

/// custom query to handle if query params are missing.
pub struct CustomQuery<T>(pub T);

#[async_trait]
impl<S, T> FromRequestParts<S> for CustomQuery<T>
where
    T: DeserializeOwned + Send + Sync,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Query::<T>::from_request_parts(parts, _state)
            .await
            .map(|Query(params)| CustomQuery(params))
            .map_err(|_| AppError::BadRequest(format!("Missing or invalid query parameters")))
    }
}

// deserialize date from YYYY-MM-DD into YYYY-MM-DDThh:mm:ssZ utc
fn deserialize_optional_date<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
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

fn deserialize_date<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
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

// --------- convert handler ---------
#[derive(Debug, Deserialize, Serialize)]
pub struct ConvertQuery {
    #[serde(rename = "from")]
    pub from: String,

    #[serde(rename = "to")]
    pub to: Currency,

    /// optional date for historical conversion
    #[serde(
        rename = "date",
        default,
        deserialize_with = "deserialize_optional_date"
    )]
    pub date: Option<DateTime<Utc>>,
}

/// GET /forex/convert
/// convert using latest or historical rates.
/// query 1: `from` money format ISO 4217 <CURRENCY_CODE> <AMOUNT>, amount may be separated by comma for thousands and dot for fractionals, e.g. ?from=USD 1,000
/// query 2: `to` currency of target conversion: e.g. ?to=USD
/// query 3(OPTIONAL); `date`(YYYY-MM-DD) for historical convert. e.g. ?date=2020-02-02
pub(crate) async fn convert_handler(
    State(ctx): State<AppContext<impl ForexStorage>>,
    CustomQuery(params): CustomQuery<ConvertQuery>,
) -> Result<impl IntoResponse, AppError> {
    match params.date {
        Some(date) => {
            let from_money = Money::from_str(&params.from)?;
            let to_currency = params.to;
            let ret =
                service::convert_historical(&ctx.forex_storage, from_money, to_currency, date)
                    .await?;

            Ok(HttpResponse::ok(ret, None))
        }
        None => {
            let from_money = Money::from_str(&params.from)?;
            let to_currency = params.to;
            let ret = service::convert(&ctx.forex_storage, from_money, to_currency).await?;

            Ok(HttpResponse::ok(ret, None))
        }
    }
}
// --------- END ---------

// --------- rates handler ---------
#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct RatesQuery {
    /// optional date for historical rates
    #[serde(
        rename = "date",
        default,
        deserialize_with = "deserialize_optional_date"
    )]
    pub date: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub(crate) struct RatesDTO {
    pub message: String,
    pub rates_date: DateTime<Utc>,
    pub rates: RatesData,
}

impl From<RatesResponse<Rates>> for RatesDTO {
    fn from(value: RatesResponse<Rates>) -> Self {
        RatesDTO {
            message: "Latest rates".to_string(),
            rates_date: value.data.latest_update,
            rates: value.data.rates,
        }
    }
}

impl From<RatesResponse<HistoricalRates>> for RatesDTO {
    fn from(value: RatesResponse<HistoricalRates>) -> Self {
        RatesDTO {
            message: "Historical rates".to_string(),
            rates_date: value.data.date,
            rates: value.data.rates,
        }
    }
}

/// GET /forex/rates
/// get latest and historical rates
/// query 1: `date`(YYYY-MM-DD) date for historical rates, e.g. ?date=2020-02-02
pub(crate) async fn get_rates_handler(
    State(ctx): State<AppContext<impl ForexStorage>>,
    CustomQuery(params): CustomQuery<RatesQuery>,
) -> Result<impl IntoResponse, AppError> {
    match params.date {
        // get historical rates
        Some(date) => Ok(HttpResponse::ok(
            RatesDTO::from(ctx.forex_storage.get_historical(date).await?),
            None,
        )),
        // get latest rates
        None => Ok(HttpResponse::ok(
            RatesDTO::from(ctx.forex_storage.get_latest().await?),
            None,
        )),
    }
}
// --------- END ---------

// --------- timeseries ---------
#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct TimeseriesQuery {
    #[serde(rename = "start", deserialize_with = "deserialize_date")]
    start: DateTime<Utc>,

    #[serde(rename = "end", deserialize_with = "deserialize_date")]
    end: DateTime<Utc>,
}

fn validate_timeseries_params(params: &TimeseriesQuery) -> Option<AppError> {
    if params.start > params.end {
        return Some(AppError::BadRequest(
            "start must not bigger than end".to_string(),
        ));
    }

    const MAX_RANGE: i64 = 5;
    const ONE_YEAR: i64 = 366;
    if params.end - params.start > Duration::days(MAX_RANGE * ONE_YEAR) {
        return Some(AppError::BadRequest(format!(
            "Max timeseries range is {} years",
            MAX_RANGE
        )));
    }

    None
}

pub(crate) async fn get_timeseries_handler(
    State(ctx): State<AppContext<impl ForexStorage>>,
    CustomQuery(params): CustomQuery<TimeseriesQuery>,
) -> Result<impl IntoResponse, AppError> {
    if let Some(err) = validate_timeseries_params(&params) {
        return Err(err);
    }
    Ok(HttpResponse::ok(
        ctx.forex_storage
            .get_historical_range(params.start, params.end)
            .await?
            .into_iter()
            .map(|rate| RatesDTO::from(rate))
            .collect::<Vec<RatesDTO>>(),
        None,
    ))
}
// --------- END ---------

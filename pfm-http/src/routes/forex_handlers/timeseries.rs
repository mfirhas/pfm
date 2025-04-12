use crate::dto::*;
use crate::global::AppContext;
use axum::{extract::State, response::IntoResponse};
use chrono::{DateTime, Duration, Utc};
use pfm_core::forex::{
    entity::{HistoricalRates, RatesData, RatesResponse},
    interface::ForexStorage,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub(crate) struct TimeseriesRatesDTO {
    pub message: String,
    pub rates_date: DateTime<Utc>,
    pub rates: RatesData,
}

impl From<RatesResponse<HistoricalRates>> for TimeseriesRatesDTO {
    fn from(value: RatesResponse<HistoricalRates>) -> Self {
        TimeseriesRatesDTO {
            message: "Timeseries rates".to_string(),
            rates_date: value.data.date,
            rates: value.data.rates,
        }
    }
}

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
            .map(|rate| TimeseriesRatesDTO::from(rate))
            .collect::<Vec<TimeseriesRatesDTO>>(),
        None,
    ))
}

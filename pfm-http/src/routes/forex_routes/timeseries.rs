use crate::dto::*;
use crate::global::AppContext;
use axum::{extract::State, response::IntoResponse};
use chrono::{DateTime, Duration, Utc};
use pfm_core::forex::{
    entity::{Rates, RatesData, RatesResponse},
    interface::{ForexHistoricalRates, ForexStorage},
};
use serde::{Deserialize, Serialize};
use tracing::instrument;

#[derive(Debug, Serialize)]
pub(crate) struct TimeseriesRatesDTO {
    pub message: String,
    pub rates_date: DateTime<Utc>,
    pub rates: RatesData,
}

impl From<RatesResponse<Rates>> for TimeseriesRatesDTO {
    fn from(value: RatesResponse<Rates>) -> Self {
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

impl Validate for TimeseriesQuery {
    fn validate(&self) -> Result<(), AppError> {
        if self.start > self.end {
            return Err(AppError::BadRequest(
                "start must not bigger than end".to_string(),
            ));
        }

        const MAX_RANGE: i64 = 5;
        const ONE_YEAR: i64 = 366;
        if self.end - self.start > Duration::days(MAX_RANGE * ONE_YEAR) {
            return Err(AppError::BadRequest(format!(
                "Max timeseries date range is {} years",
                MAX_RANGE
            )));
        }

        Ok(())
    }
}

impl BadRequestErrMsg for TimeseriesQuery {
    fn bad_request_err_msg() -> &'static str {
        "Invalid input of `start` or `end`. `start` must be in form of YYYY-MM-DD. `end` must be in form of YYYY-MM-DD."
    }
}

#[instrument(skip(ctx), ret)]
pub(crate) async fn get_timeseries_handler(
    State(ctx): State<AppContext<impl ForexStorage, impl ForexHistoricalRates>>,
    CustomQuery(params): CustomQuery<TimeseriesQuery>,
) -> Result<impl IntoResponse, AppError> {
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

use axum::{extract::State, response::IntoResponse};
use chrono::{DateTime, Utc};
use pfm_core::forex::{
    entity::{HistoricalRates, Rates, RatesData, RatesResponse},
    interface::ForexStorage,
};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::dto::*;
use crate::global::AppContext;

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

impl BadRequestErrMsg for RatesQuery {
    fn bad_request_err_msg() -> &'static str {
        "`date` is optional denoting historical rates, must be in form of YYYY-MM-DD."
    }
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

// GET /forex/rates
// get latest and historical rates
// query 1: `date`(YYYY-MM-DD) date for historical rates, e.g. ?date=2020-02-02
#[instrument(skip(ctx), ret)]
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

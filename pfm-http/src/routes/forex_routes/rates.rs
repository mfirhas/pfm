use axum::{extract::State, response::IntoResponse};
use chrono::{DateTime, Datelike, Utc};
use pfm_core::forex::{
    Currency,
    entity::{Rates, RatesData, RatesResponse},
    interface::{ForexHistoricalRates, ForexStorage},
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

impl Validate for RatesQuery {
    fn validate(&self) -> Result<(), AppError> {
        Ok(())
    }
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
    pub base: Currency,
    pub rates: RatesData,
}

impl From<RatesResponse<Rates>> for RatesDTO {
    fn from(value: RatesResponse<Rates>) -> Self {
        RatesDTO {
            message: "Successfully get rates".to_string(),
            rates_date: value.data.date,
            base: value.data.base,
            rates: value.data.rates,
        }
    }
}

// GET /forex/rates
// get latest and historical rates
// query 1: `date`(YYYY-MM-DD) date for historical rates, e.g. ?date=2020-02-02
#[instrument(skip(ctx), ret)]
pub(crate) async fn get_rates_handler(
    State(ctx): State<AppContext<impl ForexStorage, impl ForexHistoricalRates>>,
    CustomQuery(params): CustomQuery<RatesQuery>,
) -> Result<impl IntoResponse, AppError> {
    match params.date {
        // get historical rates
        Some(date) => {
            let now = Utc::now();
            if date.year() == now.year() && date.month() == now.month() && date.day() == now.day() {
                let ret = ctx.forex_storage.get_latest().await?;
                if let Some(err) = ret.error {
                    return Err(AppError::InternalServerError(err));
                }
                return Ok(HttpResponse::ok(RatesDTO::from(ret), None));
            }

            let ret = ctx.forex_storage.get_historical(date).await?;
            if let Some(err) = ret.error {
                return Err(AppError::InternalServerError(err));
            }
            Ok(HttpResponse::ok(RatesDTO::from(ret), None))
        }
        // get latest rates
        None => Ok(HttpResponse::ok(
            RatesDTO::from({
                let ret = ctx.forex_storage.get_latest().await?;
                if let Some(err) = ret.error {
                    return Err(AppError::InternalServerError(err));
                }
                ret
            }),
            None,
        )),
    }
}

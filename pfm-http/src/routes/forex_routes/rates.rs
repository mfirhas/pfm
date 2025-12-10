use std::str::FromStr;

use axum::{extract::State, response::IntoResponse};
use chrono::{DateTime, Datelike, Utc};
use pfm_core::{
    forex::{
        Currency,
        entity::{Rates, RatesData, RatesResponse},
        interface::{ForexHistoricalRates, ForexStorage},
        service,
    },
    global::constants,
};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::dto::*;
use crate::global::AppContext;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct RatesQuery {
    #[serde(rename = "base", default)]
    pub base: Option<String>,

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
    let base = if let Some(base) = params.base {
        Currency::from_str(base.as_str())?
    } else {
        constants::BASE_CURRENCY
    };

    let ret = service::get_rates(&ctx.forex_storage, base, params.date).await?;

    Ok(HttpResponse::ok(RatesDTO::from(ret), None))
}

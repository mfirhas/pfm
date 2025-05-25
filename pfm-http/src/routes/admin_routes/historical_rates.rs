use crate::{dto::*, global::AppContext};
use axum::{extract::State, response::IntoResponse};
use chrono::{DateTime, Utc};
use pfm_core::{
    forex::{
        entity::{HistoricalRates, RatesData, RatesResponse},
        interface::{ForexHistoricalRates, ForexStorage},
        Currency,
    },
    global,
};
use serde::{Deserialize, Serialize};
use tracing::instrument;

#[derive(Debug, Deserialize, Serialize)]
pub struct HistoricalRatesQuery {
    #[serde(rename = "date", default, deserialize_with = "deserialize_date")]
    pub date: DateTime<Utc>,
}

impl Validate for HistoricalRatesQuery {
    fn validate(&self) -> Result<(), AppError> {
        Ok(())
    }
}

impl BadRequestErrMsg for HistoricalRatesQuery {
    fn bad_request_err_msg() -> &'static str {
        "Date required for historical rates, format is YYYY-MM-DD"
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct HistoricalRatesDTO {
    pub message: String,
    pub rates_date: DateTime<Utc>,
    pub base: Currency,
    pub rates: RatesData,
}

impl From<RatesResponse<HistoricalRates>> for HistoricalRatesDTO {
    fn from(value: RatesResponse<HistoricalRates>) -> Self {
        HistoricalRatesDTO {
            message: "Historical rates".to_string(),
            rates_date: value.data.date,
            base: value.data.base,
            rates: value.data.rates,
        }
    }
}

/// fetch and store historical rates data from 3rd party api
#[instrument(skip(ctx), ret)]
pub(crate) async fn fetch_historical_rates_handler(
    State(ctx): State<AppContext<impl ForexStorage, impl ForexHistoricalRates>>,
    CustomQuery(params): CustomQuery<HistoricalRatesQuery>,
) -> Result<impl IntoResponse, AppError> {
    let ret = match ctx
        .forex_historical
        .historical_rates(params.date, global::constants::BASE_CURRENCY)
        .await
    {
        Ok(val) => {
            ctx.forex_storage
                .insert_historical(val.data.date, &val)
                .await?;
            Ok(HttpResponse::ok(HistoricalRatesDTO::from(val), None))
        }
        Err(error) => Err(AppError::InternalServerError(error.to_string())),
    };

    ret
}

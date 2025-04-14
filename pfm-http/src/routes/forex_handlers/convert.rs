use std::str::FromStr;

use axum::{extract::State, response::IntoResponse};
use chrono::{DateTime, Utc};
use pfm_core::forex::{interface::ForexStorage, service, Money};
use serde::{Deserialize, Serialize};

use crate::dto::*;
use crate::global::AppContext;

#[derive(Debug, Deserialize, Serialize)]
pub struct ConvertQuery {
    #[serde(rename = "from")]
    pub from: String,

    #[serde(rename = "to")]
    pub to: String,

    /// optional date for historical conversion
    #[serde(
        rename = "date",
        default,
        deserialize_with = "deserialize_optional_date"
    )]
    pub date: Option<DateTime<Utc>>,
}

impl BadRequestErrMsg for ConvertQuery {
    fn bad_request_err_msg() -> &'static str {
        r#"
        Invalid from, to, or date. 
        `from` must be in form: <CODE> <AMOUNT>, CODE is ISO 4217 standard. AMOUNT may be separated by comma for thousands, and dot for fractions. 
        `to` must be in form: <CODE>, CODE is ISO 4217 standard.
        `date` is optional denoting historical convert. Must be in form YYYY-MM-DD.
        "#
    }
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
            let to_currency = params.to.parse()?;
            let ret =
                service::convert_historical(&ctx.forex_storage, from_money, to_currency, date)
                    .await?;

            Ok(HttpResponse::ok(ret, None))
        }
        None => {
            let from_money = Money::from_str(&params.from)?;
            let to_currency = params.to.parse()?;
            let ret = service::convert(&ctx.forex_storage, from_money, to_currency).await?;

            Ok(HttpResponse::ok(ret, None))
        }
    }
}

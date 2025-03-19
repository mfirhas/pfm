// currencyapi.com
// free
// 300 reqs/month
// daily latest rates
// daily historical rates
// 10 reqs/minute

use anyhow::{anyhow, Context};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::forex::entity::RatesData;
use crate::forex::interface::{AsInternalError, ForexHistoricalRates};
use crate::forex::ForexResult;
use crate::forex::{
    entity::{HistoricalRates, RatesResponse},
    Currency, ForexError,
};

const SOURCE: &str = "currencyapi.com";

const HISTORICAL_ENDPOINT: &str = "https://api.currencyapi.com/v3/historical";

const ERROR_PREFIX: &str = "[FOREX][currencyapi.com]";

#[derive(Clone)]
pub struct Api {
    key: &'static str,
    client: reqwest::Client,
}

impl Api {
    pub fn new(api_key: &'static str, client: reqwest::Client) -> Self {
        Self {
            key: api_key,
            client,
        }
    }
}

#[derive(Debug)]
pub struct Response {
    pub base: Currency,
    pub api_response: ApiResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    #[serde(rename = "meta")]
    pub metadata: Metadata,
    #[serde(rename = "data")]
    pub rates: Data,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    #[serde(rename = "last_updated_at")]
    pub last_updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    #[serde(rename = "IDR", default)]
    pub idr: RateData,

    #[serde(rename = "USD", default)]
    pub usd: RateData,

    #[serde(rename = "EUR", default)]
    pub eur: RateData,

    #[serde(rename = "GBP", default)]
    pub gbp: RateData,

    #[serde(rename = "JPY", default)]
    pub jpy: RateData,

    #[serde(rename = "CHF", default)]
    pub chf: RateData,

    #[serde(rename = "SGD", default)]
    pub sgd: RateData,

    #[serde(rename = "CNY", default)]
    pub cny: RateData,

    #[serde(rename = "SAR", default)]
    pub sar: RateData,

    #[serde(rename = "XAU", default)]
    pub xau: RateData,

    #[serde(rename = "XAG", default)]
    pub xag: RateData,

    #[serde(rename = "XPT", default)]
    pub xpt: RateData,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RateData {
    #[serde(rename = "code")]
    pub code: String,
    #[serde(rename = "value")]
    pub value: Decimal,
}

impl TryFrom<Response> for RatesResponse<HistoricalRates> {
    type Error = ForexError;

    fn try_from(value: Response) -> Result<Self, Self::Error> {
        let date = value
            .api_response
            .metadata
            .last_updated_at
            .parse::<DateTime<Utc>>()
            .context("currency_api parsing datetime")
            .as_internal_err()?;

        let historical_rates = HistoricalRates {
            date,
            base: value.base,
            rates: RatesData {
                idr: value.api_response.rates.idr.value,
                usd: value.api_response.rates.usd.value,
                eur: value.api_response.rates.eur.value,
                gbp: value.api_response.rates.gbp.value,
                jpy: value.api_response.rates.jpy.value,
                chf: value.api_response.rates.chf.value,
                sgd: value.api_response.rates.sgd.value,
                cny: value.api_response.rates.cny.value,
                sar: value.api_response.rates.sar.value,
                xau: value.api_response.rates.xau.value,
                xag: value.api_response.rates.xag.value,
                xpt: value.api_response.rates.xpt.value,
            },
        };

        Ok(RatesResponse::new(SOURCE.into(), historical_rates))
    }
}

#[async_trait]
impl ForexHistoricalRates for Api {
    async fn historical_rates(
        &self,
        date: chrono::DateTime<chrono::Utc>,
        base: Currency,
    ) -> ForexResult<RatesResponse<HistoricalRates>> {
        let yyyymmdd = date.format("%Y-%m-%d").to_string();

        let params = [
            ("apikey", self.key),
            ("base_currency", base.code()),
            ("date", yyyymmdd.as_str()),
            (
                "currencies",
                "IDR,USD,EUR,GBP,JPY,CHF,SGD,CNY,SAR,XAU,XAG,XPT",
            ),
        ];

        let ret = self
            .client
            .get(HISTORICAL_ENDPOINT)
            .query(&params)
            .send()
            .await
            .context("invoking currency_api historical rates")
            .as_internal_err()?
            .text()
            .await
            .context("fetch currency_api historical response as string")
            .as_internal_err()?;

        let resp = serde_json::from_str::<ApiResponse>(&ret)
            .context("currency_api parsing into json")
            .as_internal_err()?;

        let resp = Response {
            base,
            api_response: resp,
        };

        Ok(resp.try_into()?)
    }
}

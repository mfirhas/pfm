// currencyapi.com
// free
// 300 reqs/month
// daily latest rates
// daily historical rates
// 10 reqs/minute

use crate::forex::{Currencies, ForexHistoricalRates, HistoricalRates, RatesResponse};
use anyhow::anyhow;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

const SOURCE: &str = "currencyapi.com";

const HISTORICAL_ENDPOINT: &str = "https://api.currencyapi.com/v3/historical";

const ERROR_PREFIX: &str = "[FOREX][currency-api]";

pub(crate) struct Api {
    key: &'static str,
    client: reqwest::Client,
}

impl Api {
    pub(crate) fn new(api_key: &'static str, client: reqwest::Client) -> Self {
        Self {
            key: api_key,
            client,
        }
    }
}

#[derive(Debug)]
pub struct Response {
    pub base: Currencies,
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
    #[serde(rename = "IDR")]
    pub idr: Currency,

    #[serde(rename = "USD")]
    pub usd: Currency,

    #[serde(rename = "EUR")]
    pub eur: Currency,

    #[serde(rename = "GBP")]
    pub gbp: Currency,

    #[serde(rename = "JPY")]
    pub jpy: Currency,

    #[serde(rename = "CHF")]
    pub chf: Currency,

    #[serde(rename = "SGD")]
    pub sgd: Currency,

    #[serde(rename = "CNY")]
    pub cny: Currency,

    #[serde(rename = "SAR")]
    pub sar: Currency,

    #[serde(rename = "XAU")]
    pub xau: Currency,

    #[serde(rename = "XAG")]
    pub xag: Currency,

    #[serde(rename = "XPT")]
    pub xpt: Currency,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Currency {
    #[serde(rename = "code")]
    pub code: String,
    #[serde(rename = "value")]
    pub value: Decimal,
}

impl TryFrom<Response> for crate::forex::RatesResponse<crate::forex::HistoricalRates> {
    type Error = anyhow::Error;

    fn try_from(value: Response) -> Result<Self, Self::Error> {
        let date = value
            .api_response
            .metadata
            .last_updated_at
            .parse::<DateTime<Utc>>()
            .map_err(|err| anyhow!("{} Failed parsing datetime: {}", ERROR_PREFIX, err))?;

        let historical_rates = HistoricalRates {
            date,
            base: value.base,
            rates: crate::forex::RatesData {
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
        base: Currencies,
    ) -> crate::forex::ForexResult<crate::forex::RatesResponse<HistoricalRates>> {
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

        let ret: ApiResponse = self
            .client
            .get(HISTORICAL_ENDPOINT)
            .query(&params)
            .send()
            .await?
            .error_for_status()
            .map_err(|err| {
                anyhow!(
                    "{} failed calling api historical rates: {}",
                    ERROR_PREFIX,
                    err
                )
            })?
            .json()
            .await
            .map_err(|err| {
                anyhow!(
                    "{} failed parsing historical rates result into json: {}",
                    ERROR_PREFIX,
                    err
                )
            })?;

        let resp = Response {
            base,
            api_response: ret,
        };

        Ok(resp.try_into()?)
    }
}

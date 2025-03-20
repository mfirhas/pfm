// currencyapi.com
// free
// 300 reqs/month
// daily latest rates
// daily historical rates
// 10 reqs/minute

use anyhow::Context;
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
    #[serde(rename = "USD", default)]
    pub usd: RateData,

    #[serde(rename = "CAD", default)]
    pub cad: RateData,

    #[serde(rename = "EUR", default)]
    pub eur: RateData,

    #[serde(rename = "GBP", default)]
    pub gbp: RateData,

    #[serde(rename = "CHF", default)]
    pub chf: RateData,

    #[serde(rename = "RUB", default)]
    pub rub: RateData,

    #[serde(rename = "CNY", default)]
    pub cny: RateData,

    #[serde(rename = "JPY", default)]
    pub jpy: RateData,

    #[serde(rename = "KRW", default)]
    pub krw: RateData,

    #[serde(rename = "HKD", default)]
    pub hkd: RateData,

    #[serde(rename = "IDR", default)]
    pub idr: RateData,

    #[serde(rename = "MYR", default)]
    pub myr: RateData,

    #[serde(rename = "SGD", default)]
    pub sgd: RateData,

    #[serde(rename = "THB", default)]
    pub thb: RateData,

    #[serde(rename = "SAR", default)]
    pub sar: RateData,

    #[serde(rename = "AED", default)]
    pub aed: RateData,

    #[serde(rename = "KWD", default)]
    pub kwd: RateData,

    #[serde(rename = "INR", default)]
    pub inr: RateData,

    #[serde(rename = "AUD", default)]
    pub aud: RateData,

    #[serde(rename = "NZD", default)]
    pub nzd: RateData,

    #[serde(rename = "XAU", default)]
    pub xau: RateData,

    #[serde(rename = "XAG", default)]
    pub xag: RateData,

    #[serde(rename = "XPT", default)]
    pub xpt: RateData,

    #[serde(rename = "XPD", default)]
    pub xpd: RateData,

    #[serde(rename = "XRH", default)]
    pub xrh: RateData,

    #[serde(rename = "BTC", default)]
    pub btc: RateData,

    #[serde(rename = "ETH", default)]
    pub eth: RateData,

    #[serde(rename = "SOL", default)]
    pub sol: RateData,

    #[serde(rename = "XRP", default)]
    pub xrp: RateData,

    #[serde(rename = "ADA", default)]
    pub ada: RateData,
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
                usd: value.api_response.rates.usd.value,
                cad: value.api_response.rates.cad.value,
                eur: value.api_response.rates.eur.value,
                gbp: value.api_response.rates.gbp.value,
                chf: value.api_response.rates.chf.value,
                rub: value.api_response.rates.rub.value,
                cny: value.api_response.rates.cny.value,
                jpy: value.api_response.rates.jpy.value,
                krw: value.api_response.rates.krw.value,
                hkd: value.api_response.rates.hkd.value,
                idr: value.api_response.rates.idr.value,
                myr: value.api_response.rates.myr.value,
                sgd: value.api_response.rates.sgd.value,
                thb: value.api_response.rates.thb.value,
                sar: value.api_response.rates.sar.value,
                aed: value.api_response.rates.aed.value,
                kwd: value.api_response.rates.kwd.value,
                inr: value.api_response.rates.inr.value,
                aud: value.api_response.rates.aud.value,
                nzd: value.api_response.rates.nzd.value,
                xau: value.api_response.rates.xau.value,
                xag: value.api_response.rates.xag.value,
                xpt: value.api_response.rates.xpt.value,
                xpd: value.api_response.rates.xpd.value,
                xrh: value.api_response.rates.xrh.value,
                btc: value.api_response.rates.btc.value,
                eth: value.api_response.rates.eth.value,
                sol: value.api_response.rates.sol.value,
                xrp: value.api_response.rates.xrp.value,
                ada: value.api_response.rates.ada.value,
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

        let currencies = Currency::to_comma_separated_list_str();

        let params = [
            ("apikey", self.key),
            ("base_currency", base.code()),
            ("date", yyyymmdd.as_str()),
            ("currencies", &currencies),
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

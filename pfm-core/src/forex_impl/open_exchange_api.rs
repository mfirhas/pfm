// openexchangerates.org
// Hourly rate updates
// Daily historical data
// 1,000 API requests per month
// rate limit: 5 requests / second
// might return 429 once reach limit.
// Get historical exchange rates for any date available from the Open Exchange Rates API, currently going back to 1st January 1999.
// gold price start exist on 2013-04-01

use anyhow::anyhow;
use std::str::FromStr;

use crate::error::AsInternalError;
use crate::forex::{
    entity::{HistoricalRates, RatesData, RatesResponse},
    interface::{ForexHistoricalRates, ForexRates},
    Currency, ForexError, ForexResult,
};
use anyhow::Context;
use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

const SOURCE: &str = "openexchangerates.org";

const ERROR_PREFIX: &str = "[FOREX][openexchangerates.org]";

const LATEST_ENDPOINT: &str = "https://openexchangerates.org/api/latest.json";

// :date = YYYY-MM-DD
const HISTORICAL_ENDPOINT: &str = "https://openexchangerates.org/api/historical/:date.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    #[serde(rename = "disclaimer")]
    pub disclaimer: String,

    #[serde(rename = "license")]
    pub license: String,

    #[serde(rename = "timestamp")]
    pub timestamp: i64,

    #[serde(rename = "base")]
    pub base_currency: String,

    #[serde(rename = "rates")]
    pub rates: Rates,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rates {
    #[serde(rename = "USD", default)]
    pub usd: Decimal,

    #[serde(rename = "CAD", default)]
    pub cad: Decimal,

    #[serde(rename = "EUR", default)]
    pub eur: Decimal,

    #[serde(rename = "GBP", default)]
    pub gbp: Decimal,

    #[serde(rename = "CHF", default)]
    pub chf: Decimal,

    #[serde(rename = "RUB", default)]
    pub rub: Decimal,

    #[serde(rename = "CNY", default)]
    pub cny: Decimal,

    #[serde(rename = "JPY", default)]
    pub jpy: Decimal,

    #[serde(rename = "KRW", default)]
    pub krw: Decimal,

    #[serde(rename = "HKD", default)]
    pub hkd: Decimal,

    #[serde(rename = "IDR", default)]
    pub idr: Decimal,

    #[serde(rename = "MYR", default)]
    pub myr: Decimal,

    #[serde(rename = "SGD", default)]
    pub sgd: Decimal,

    #[serde(rename = "THB", default)]
    pub thb: Decimal,

    #[serde(rename = "SAR", default)]
    pub sar: Decimal,

    #[serde(rename = "AED", default)]
    pub aed: Decimal,

    #[serde(rename = "KWD", default)]
    pub kwd: Decimal,

    #[serde(rename = "INR", default)]
    pub inr: Decimal,

    #[serde(rename = "AUD", default)]
    pub aud: Decimal,

    #[serde(rename = "NZD", default)]
    pub nzd: Decimal,

    #[serde(rename = "XAU", default)]
    pub xau: Decimal,

    #[serde(rename = "XAG", default)]
    pub xag: Decimal,

    #[serde(rename = "XPT", default)]
    pub xpt: Decimal,

    #[serde(rename = "BTC", default)]
    pub btc: Decimal,

    #[serde(rename = "ETH", default)]
    pub eth: Decimal,

    #[serde(rename = "SOL", default)]
    pub sol: Decimal,

    #[serde(rename = "XRP", default)]
    pub xrp: Decimal,

    #[serde(rename = "ADA", default)]
    pub ada: Decimal,
}

impl TryFrom<Response> for RatesResponse<crate::forex::entity::Rates> {
    type Error = ForexError;

    fn try_from(value: Response) -> Result<Self, Self::Error> {
        let date =
            Utc.timestamp_opt(value.timestamp, 0)
                .single()
                .ok_or(ForexError::internal_error(
                    "openexchangerates converting latest rates unix epoch to utc",
                ))?;

        let rates = RatesData {
            usd: value.rates.usd,
            cad: value.rates.cad,
            eur: value.rates.eur,
            gbp: value.rates.gbp,
            chf: value.rates.chf,
            rub: value.rates.rub,
            cny: value.rates.cny,
            jpy: value.rates.jpy,
            krw: value.rates.krw,
            hkd: value.rates.hkd,
            idr: value.rates.idr,
            myr: value.rates.myr,
            sgd: value.rates.sgd,
            thb: value.rates.thb,
            sar: value.rates.sar,
            aed: value.rates.aed,
            kwd: value.rates.kwd,
            inr: value.rates.inr,
            aud: value.rates.aud,
            nzd: value.rates.nzd,
            xau: value.rates.xau,
            xag: value.rates.xag,
            xpt: value.rates.xpt,
            btc: value.rates.btc,
            eth: value.rates.eth,
            sol: value.rates.sol,
            xrp: value.rates.xrp,
            ada: value.rates.ada,
        };

        let base = Currency::from_str(&value.base_currency)
            .context("openexchangerates parse base currency")
            .as_internal_err()?;

        let ret = crate::forex::entity::Rates {
            latest_update: date,
            base,
            rates,
        };

        Ok(RatesResponse::new(SOURCE.into(), ret))
    }
}

impl TryFrom<Response> for RatesResponse<HistoricalRates> {
    type Error = ForexError;

    fn try_from(value: Response) -> Result<Self, Self::Error> {
        let date =
            Utc.timestamp_opt(value.timestamp, 0)
                .single()
                .ok_or(ForexError::internal_error(
                    "openexchangerates converting historical rates unix epoch to utc",
                ))?;

        let rates = RatesData {
            usd: value.rates.usd,
            cad: value.rates.cad,
            eur: value.rates.eur,
            gbp: value.rates.gbp,
            chf: value.rates.chf,
            rub: value.rates.rub,
            cny: value.rates.cny,
            jpy: value.rates.jpy,
            krw: value.rates.krw,
            hkd: value.rates.hkd,
            idr: value.rates.idr,
            myr: value.rates.myr,
            sgd: value.rates.sgd,
            thb: value.rates.thb,
            sar: value.rates.sar,
            aed: value.rates.aed,
            kwd: value.rates.kwd,
            inr: value.rates.inr,
            aud: value.rates.aud,
            nzd: value.rates.nzd,
            xau: value.rates.xau,
            xag: value.rates.xag,
            xpt: value.rates.xpt,
            btc: value.rates.btc,
            eth: value.rates.eth,
            sol: value.rates.sol,
            xrp: value.rates.xrp,
            ada: value.rates.ada,
        };

        let base = Currency::from_str(&value.base_currency)
            .context("openexchangerates parse base currency")
            .as_internal_err()?;

        let ret = crate::forex::entity::HistoricalRates { base, date, rates };

        Ok(RatesResponse::new(SOURCE.into(), ret))
    }
}

// status usage
#[derive(Debug, Serialize, Deserialize)]
pub struct StatusResponse {
    pub data: Data,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    pub usage: Usage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Usage {
    pub requests: u32,
    pub requests_quota: u32,
    pub requests_remaining: u32,
}

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

    pub async fn status(&self) -> ForexResult<StatusResponse> {
        let endpoint = "https://openexchangerates.org/api/usage.json";
        let params = [("app_id", self.key)];

        let status: StatusResponse = self
            .client
            .get(endpoint)
            .query(&params)
            .send()
            .await
            .context("openexchangerates invoke status")
            .as_internal_err()?
            .json::<StatusResponse>()
            .await
            .context("openexchangerates parsing to json")
            .as_internal_err()?;

        Ok(status)
    }
}

#[async_trait]
impl ForexRates for Api {
    async fn rates(
        &self,
        base: Currency,
    ) -> crate::forex::ForexResult<RatesResponse<crate::forex::entity::Rates>> {
        let symbols = Currency::to_comma_separated_list_str();

        let params = [
            ("app_id", self.key),
            ("base", base.code()),
            ("symbols", &symbols),
        ];

        let ret = self
            .client
            .get(LATEST_ENDPOINT)
            .query(&params)
            .send()
            .await
            .context("openexchangerates invoke latest rates api")
            .as_internal_err()?
            .text()
            .await
            .context("openexchangerates fetch latest rates api")
            .as_internal_err()?;

        let resp = serde_json::from_str::<Response>(&ret)
            .map_err(|err| {
                anyhow!(
                    "open_exchange_api parsing latest rates into json, error parsing: {}, \n Caused by: {}",
                    &ret,
                    err
                )
            })
            .as_internal_err()?;

        Ok(resp.try_into()?)
    }
}

#[async_trait]
impl ForexHistoricalRates for Api {
    async fn historical_rates(
        &self,
        date: chrono::DateTime<chrono::Utc>,
        base: Currency,
    ) -> crate::forex::ForexResult<RatesResponse<crate::forex::entity::HistoricalRates>> {
        let yyyymmdd = date.format("%Y-%m-%d").to_string();
        let endpoint = HISTORICAL_ENDPOINT.replace(":date", yyyymmdd.as_str());

        let symbols = Currency::to_comma_separated_list_str();

        let params = [
            ("app_id", self.key),
            ("base", base.code()),
            ("symbols", &symbols),
        ];

        let ret = self
            .client
            .get(&endpoint)
            .query(&params)
            .send()
            .await
            .context("openexchangerates invoke historical rates api")
            .as_internal_err()?
            .text()
            .await
            .context("openexchangerates fetch historical rates to json")
            .as_internal_err()?;

        let resp = serde_json::from_str::<Response>(&ret).map_err(|err| {
                anyhow!(
                    "open_exchange_api parsing historical rates into json, error parsing: {}, \n Caused by: {}",
                    &ret,
                    err
                )
            })
            .as_internal_err()?;

        Ok(resp.try_into()?)
    }
}

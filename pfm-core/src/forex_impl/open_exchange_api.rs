// openexchangerates.org
// Hourly rate updates
// Daily historical data
// 1,000 API requests per month

use std::str::FromStr;

use crate::forex::ForexError::{self, OpenExchangeAPIError};
use crate::forex::{
    Currencies, ForexHistoricalRates, ForexRates, HistoricalRates, RatesData, RatesResponse,
};
use anyhow::anyhow;
use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

const SOURCE: &str = "openexchangerates.org";

const ERROR_PREFIX: &str = "[FOREX][open-exchange-api]";

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
    #[serde(rename = "CHF")]
    pub chf: Decimal,

    #[serde(rename = "CNY")]
    pub cny: Decimal,

    #[serde(rename = "EUR")]
    pub eur: Decimal,

    #[serde(rename = "GBP")]
    pub gbp: Decimal,

    #[serde(rename = "IDR")]
    pub idr: Decimal,

    #[serde(rename = "JPY")]
    pub jpy: Decimal,

    #[serde(rename = "SAR")]
    pub sar: Decimal,

    #[serde(rename = "SGD")]
    pub sgd: Decimal,

    #[serde(rename = "USD")]
    pub usd: Decimal,

    #[serde(rename = "XAG")]
    pub xag: Decimal,

    #[serde(rename = "XAU")]
    pub xau: Decimal,

    #[serde(rename = "XPT")]
    pub xpt: Decimal,
}

impl TryFrom<Response> for RatesResponse<crate::forex::Rates> {
    type Error = ForexError;

    fn try_from(value: Response) -> Result<Self, Self::Error> {
        let date = Utc
            .timestamp_opt(value.timestamp, 0)
            .single()
            .ok_or(OpenExchangeAPIError(anyhow!(
                "{} Failed converting unix epoch {} into utc",
                ERROR_PREFIX,
                value.timestamp,
            )))?;

        let rates = RatesData {
            idr: value.rates.idr,
            usd: value.rates.usd,
            eur: value.rates.eur,
            gbp: value.rates.gbp,
            jpy: value.rates.jpy,
            chf: value.rates.chf,
            sgd: value.rates.sgd,
            cny: value.rates.cny,
            sar: value.rates.sar,
            xau: value.rates.xau,
            xag: value.rates.xag,
            xpt: value.rates.xpt,
        };

        let base = Currencies::from_str(&value.base_currency).map_err(|err| {
            OpenExchangeAPIError(anyhow!(
                "{} base currency not supported :{}",
                ERROR_PREFIX,
                err
            ))
        })?;

        let ret = crate::forex::Rates {
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
        let date = Utc
            .timestamp_opt(value.timestamp, 0)
            .single()
            .ok_or(OpenExchangeAPIError(anyhow!(
                "{} Failed converting unix epoch {} into utc",
                ERROR_PREFIX,
                value.timestamp
            )))?;

        let rates = RatesData {
            idr: value.rates.idr,
            usd: value.rates.usd,
            eur: value.rates.eur,
            gbp: value.rates.gbp,
            jpy: value.rates.jpy,
            chf: value.rates.chf,
            sgd: value.rates.sgd,
            cny: value.rates.cny,
            sar: value.rates.sar,
            xau: value.rates.xau,
            xag: value.rates.xag,
            xpt: value.rates.xpt,
        };

        let base = Currencies::from_str(&value.base_currency).map_err(|err| {
            OpenExchangeAPIError(anyhow!(
                "{} base currency not supported :{}",
                ERROR_PREFIX,
                err
            ))
        })?;

        let ret = crate::forex::HistoricalRates { base, date, rates };

        Ok(RatesResponse::new(SOURCE.into(), ret))
    }
}

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

#[async_trait]
impl ForexRates for Api {
    async fn rates(
        &self,
        base: Currencies,
    ) -> crate::forex::ForexResult<RatesResponse<crate::forex::Rates>> {
        let params = [
            ("app_id", self.key),
            ("base", base.code()),
            ("symbols", "IDR,USD,EUR,GBP,JPY,CHF,SGD,CNY,SAR,XAU,XAG,XPT"),
        ];

        let ret: Response = self
            .client
            .get(LATEST_ENDPOINT)
            .query(&params)
            .send()
            .await
            .map_err(|err| {
                OpenExchangeAPIError(anyhow!(
                    "{} failed calling api rates: {}",
                    ERROR_PREFIX,
                    err
                ))
            })?
            .error_for_status()
            .map_err(|err| {
                OpenExchangeAPIError(anyhow!(
                    "{} failed because non 200/201 status code on api rates: {}",
                    ERROR_PREFIX,
                    err
                ))
            })?
            .json()
            .await
            .map_err(|err| {
                OpenExchangeAPIError(anyhow!(
                    "{} failed parsing rates result into json: {}",
                    ERROR_PREFIX,
                    err
                ))
            })?;

        Ok(ret.try_into()?)
    }
}

#[async_trait]
impl ForexHistoricalRates for Api {
    async fn historical_rates(
        &self,
        date: chrono::DateTime<chrono::Utc>,
        base: Currencies,
    ) -> crate::forex::ForexResult<RatesResponse<crate::forex::HistoricalRates>> {
        let yyyymmdd = date.format("%Y-%m-%d").to_string();
        let endpoint = HISTORICAL_ENDPOINT.replace(":date", yyyymmdd.as_str());
        let params = [
            ("app_id", self.key),
            ("base", base.code()),
            ("symbols", "IDR,USD,EUR,GBP,JPY,CHF,SGD,CNY,SAR,XAU,XAG,XPT"),
        ];

        let ret: Response = self
            .client
            .get(&endpoint)
            .query(&params)
            .send()
            .await
            .map_err(|err| {
                OpenExchangeAPIError(anyhow!(
                    "{} failed calling api historical rates: {}",
                    ERROR_PREFIX,
                    err
                ))
            })?
            .error_for_status()
            .map_err(|err| {
                OpenExchangeAPIError(anyhow!(
                    "{} failed because non 200/201 status code on api historical rates: {}",
                    ERROR_PREFIX,
                    err
                ))
            })?
            .json()
            .await
            .map_err(|err| {
                OpenExchangeAPIError(anyhow!(
                    "{} failed parsing historical rates result into json: {}",
                    ERROR_PREFIX,
                    err
                ))
            })?;

        Ok(ret.try_into()?)
    }
}

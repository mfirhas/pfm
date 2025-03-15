// This API fetched from https://github.com/fawazahmed0/exchange-api
// Docs: https://github.com/fawazahmed0/exchange-api/blob/main/README.md
// Endpoint are:
// - https://cdn.jsdelivr.net/npm/@fawazahmed0/currency-api@latest/v1/currencies.json
// - https://latest.currency-api.pages.dev/v1/currencies/eur.json
// API data updated at every 00:00 UTC. Source: https://github.com/fawazahmed0/exchange-api/issues/41
// prefer cloudflare api for updated uncached data. https://github.com/fawazahmed0/exchange-api/issues/96
// specs:
// + totally free
// - DAILY updates at 00.00 UTC, but slower to update on time.
// - very limited historical rates.

use crate::forex::{
    entity::{HistoricalRates, RatesData, RatesResponse},
    interface::{ForexHistoricalRates, ForexRates},
    Currency,
    ForexError::{self, APIError},
    ForexResult,
};
use anyhow::anyhow;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

const SOURCE: &str = "https://github.com/fawazahmed0/exchange-api/";

/// Endpoint for uncached data.
/// @date format is YYYY-MM-DD
const CLOUDFLARE_ENDPOINT_V1: &str =
    "https://{date}.currency-api.pages.dev/v1/currencies/{currency_code}.json";

const ERROR_PREFIX: &str = "[FOREX][https://github.com/fawazahmed0/exchange-api/]";

#[derive(Debug)]
pub struct Response {
    base: Currency,
    api_response: ApiResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    date: String,
    #[serde(flatten)]
    rates: Rates,
}

impl TryFrom<Response> for RatesResponse<crate::forex::entity::Rates> {
    type Error = ForexError;

    fn try_from(value: Response) -> Result<Self, Self::Error> {
        let utc = format!("{}T00:00:00Z", value.api_response.date);
        let date = utc.parse::<DateTime<Utc>>().map_err(|err| {
            APIError(anyhow!(
                "{} Failed converting datetime in ExchangeAPIResponse into crate::forex::Rates : {}",
                ERROR_PREFIX,
                err
            ))
        })?;
        let forex_rates = crate::forex::entity::Rates {
            latest_update: date,
            base: value.base,
            rates: RatesData {
                idr: value.api_response.rates.currencies().idr,
                usd: value.api_response.rates.currencies().usd,
                eur: value.api_response.rates.currencies().eur,
                gbp: value.api_response.rates.currencies().gbp,
                jpy: value.api_response.rates.currencies().jpy,
                chf: value.api_response.rates.currencies().chf,
                sgd: value.api_response.rates.currencies().sgd,
                cny: value.api_response.rates.currencies().cny,
                sar: value.api_response.rates.currencies().sar,
                xau: value.api_response.rates.currencies().xau,
                xag: value.api_response.rates.currencies().xag,
                xpt: value.api_response.rates.currencies().xpt,
            },
        };

        Ok(RatesResponse::new(SOURCE.into(), forex_rates))
    }
}

impl TryFrom<Response> for RatesResponse<HistoricalRates> {
    type Error = ForexError;

    fn try_from(value: Response) -> Result<Self, Self::Error> {
        let utc = format!("{}T00:00:00Z", value.api_response.date);
        let date = utc.parse::<DateTime<Utc>>().map_err(|err| {
            APIError(anyhow!(
                "{} Failed converting datetime in ExchangeAPIResponse into crate::forex::Rates : {}",
                ERROR_PREFIX,
                err
            ))
        })?;
        let forex_rates = HistoricalRates {
            date,
            base: value.base,
            rates: RatesData {
                idr: value.api_response.rates.currencies().idr,
                usd: value.api_response.rates.currencies().usd,
                eur: value.api_response.rates.currencies().eur,
                gbp: value.api_response.rates.currencies().gbp,
                jpy: value.api_response.rates.currencies().jpy,
                chf: value.api_response.rates.currencies().chf,
                sgd: value.api_response.rates.currencies().sgd,
                cny: value.api_response.rates.currencies().cny,
                sar: value.api_response.rates.currencies().sar,
                xau: value.api_response.rates.currencies().xau,
                xag: value.api_response.rates.currencies().xag,
                xpt: value.api_response.rates.currencies().xpt,
            },
        };

        Ok(RatesResponse::new(SOURCE.into(), forex_rates))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Rates {
    #[serde(rename = "idr")]
    IDR(RatesData),

    #[serde(rename = "usd")]
    USD(RatesData),

    #[serde(rename = "eur")]
    EUR(RatesData),

    #[serde(rename = "gbp")]
    GBP(RatesData),

    #[serde(rename = "jpy")]
    JPY(RatesData),

    #[serde(rename = "chf")]
    CHF(RatesData),

    #[serde(rename = "sgd")]
    SGD(RatesData),

    #[serde(rename = "cny")]
    CNY(RatesData),

    #[serde(rename = "sar")]
    SAR(RatesData),

    #[serde(rename = "xau")]
    XAU(RatesData),

    #[serde(rename = "xag")]
    XAG(RatesData),

    #[serde(rename = "xpt")]
    XPT(RatesData),
}

impl Rates {
    pub fn currencies(&self) -> &RatesData {
        match self {
            Rates::IDR(currencies) => currencies,
            Rates::USD(currencies) => currencies,
            Rates::EUR(currencies) => currencies,
            Rates::GBP(currencies) => currencies,
            Rates::JPY(currencies) => currencies,
            Rates::CHF(currencies) => currencies,
            Rates::SGD(currencies) => currencies,
            Rates::CNY(currencies) => currencies,
            Rates::SAR(currencies) => currencies,
            Rates::XAU(currencies) => currencies,
            Rates::XAG(currencies) => currencies,
            Rates::XPT(currencies) => currencies,
        }
    }
}

#[derive(Clone)]
pub struct Api {
    client: reqwest::Client,
}

impl Api {
    pub fn new(client: reqwest::Client) -> Self {
        Api { client }
    }
}

#[async_trait]
impl ForexRates for Api {
    async fn rates(
        &self,
        base: Currency,
    ) -> ForexResult<RatesResponse<crate::forex::entity::Rates>> {
        let endpoint = CLOUDFLARE_ENDPOINT_V1
            .replace("{date}", "latest")
            .replace("{currency_code}", base.code().to_lowercase().as_str());

        let ret: ApiResponse = self
            .client
            .get(&endpoint)
            .send()
            .await
            .map_err(|err| {
                APIError(anyhow!(
                    "{} failed calling api rates: {}",
                    ERROR_PREFIX,
                    err
                ))
            })?
            .error_for_status()
            .map_err(|err| {
                APIError(anyhow!(
                    "{} failed because non 200/201 status code on api rates: {}",
                    ERROR_PREFIX,
                    err
                ))
            })?
            .json()
            .await
            .map_err(|err| {
                APIError(anyhow!(
                    "{} failed parsing rates result into json: {}",
                    ERROR_PREFIX,
                    err
                ))
            })?;

        let ret = Response {
            api_response: ret,
            base,
        };

        Ok(ret.try_into()?)
    }
}

#[async_trait]
impl ForexHistoricalRates for Api {
    async fn historical_rates(
        &self,
        date: chrono::DateTime<chrono::Utc>,
        base: Currency,
    ) -> crate::forex::ForexResult<RatesResponse<HistoricalRates>> {
        let yyyymmdd = date.format("%Y-%m-%d").to_string();
        let endpoint = CLOUDFLARE_ENDPOINT_V1
            .replace("{date}", &yyyymmdd)
            .replace("{currency_code}", base.code().to_lowercase().as_str());

        let ret: ApiResponse = self
            .client
            .get(&endpoint)
            .send()
            .await
            .map_err(|err| {
                APIError(anyhow!(
                    "{} failed calling api historical rates: {}",
                    ERROR_PREFIX,
                    err
                ))
            })?
            .error_for_status()
            .map_err(|err| {
                APIError(anyhow!(
                    "{} failed because non 200/201 status code on api historical rates: {}",
                    ERROR_PREFIX,
                    err
                ))
            })?
            .json()
            .await
            .map_err(|err| {
                APIError(anyhow!(
                    "{} failed parsing historical rates result into json: {}",
                    ERROR_PREFIX,
                    err
                ))
            })?;

        let ret = Response {
            base,
            api_response: ret,
        };

        Ok(ret.try_into()?)
    }
}

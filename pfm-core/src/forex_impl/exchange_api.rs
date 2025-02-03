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

use crate::forex::{Currencies, ForexHistoricalRates, ForexRates};
use anyhow::anyhow;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Endpoint for uncached data.
/// @date format is YYYY-MM-DD
const CLOUDFLARE_ENDPOINT_V1: &str =
    "https://{date}.currency-api.pages.dev/v1/currencies/{currency_code}.json";

const ERROR_PREFIX: &str = "[FOREX][exchange-api]";

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    date: String,
    #[serde(flatten)]
    rates: Rates,
}

impl TryFrom<Response> for crate::forex::Rates {
    type Error = anyhow::Error;

    fn try_from(value: Response) -> Result<Self, Self::Error> {
        let utc = format!("{}T00:00:00Z", value.date);
        let date = utc.parse::<DateTime<Utc>>().map_err(|err| {
            anyhow!(
                "{} Failed converting datetime in ExchangeAPIResponse into crate::forex::Rates : {}",
                ERROR_PREFIX,
                err
            )
        })?;
        let forex_rates = crate::forex::Rates {
            date,
            rates: Currencies {
                idr: value.rates.currencies().idr,
                usd: value.rates.currencies().usd,
                eur: value.rates.currencies().eur,
                gbp: value.rates.currencies().gbp,
                jpy: value.rates.currencies().jpy,
                chf: value.rates.currencies().chf,
                sgd: value.rates.currencies().sgd,
                cny: value.rates.currencies().cny,
                sar: value.rates.currencies().sar,
                xau: value.rates.currencies().xau,
                xag: value.rates.currencies().xag,
                xpt: value.rates.currencies().xpt,
            },
        };

        Ok(forex_rates)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Rates {
    #[serde(rename = "idr")]
    IDR(Currencies),

    #[serde(rename = "usd")]
    USD(Currencies),

    #[serde(rename = "eur")]
    EUR(Currencies),

    #[serde(rename = "gbp")]
    GBP(Currencies),

    #[serde(rename = "jpy")]
    JPY(Currencies),

    #[serde(rename = "chf")]
    CHF(Currencies),

    #[serde(rename = "sgd")]
    SGD(Currencies),

    #[serde(rename = "cny")]
    CNY(Currencies),

    #[serde(rename = "sar")]
    SAR(Currencies),

    #[serde(rename = "xau")]
    XAU(Currencies),

    #[serde(rename = "xag")]
    XAG(Currencies),

    #[serde(rename = "xpt")]
    XPT(Currencies),
}

impl Rates {
    pub fn currencies(&self) -> &Currencies {
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

pub(crate) struct Api<'CLIENT> {
    client: &'CLIENT reqwest::Client,
}

impl<'CLIENT> Api<'CLIENT> {
    pub fn new(client: &'CLIENT reqwest::Client) -> Self {
        Api { client }
    }
}

#[async_trait]
impl ForexRates for Api<'_> {
    async fn rates(
        &self,
        base: iso_currency::Currency,
    ) -> crate::forex::ForexResult<crate::forex::Rates> {
        let endpoint = CLOUDFLARE_ENDPOINT_V1
            .replace("{date}", "latest")
            .replace("{currency_code}", base.code().to_lowercase().as_str());

        let ret: Response = self
            .client
            .get(&endpoint)
            .send()
            .await?
            .error_for_status()
            .map_err(|err| anyhow!("{} failed calling api rates: {}", ERROR_PREFIX, err))?
            .json()
            .await
            .map_err(|err| {
                anyhow!(
                    "{} failed parsing rates result into json: {}",
                    ERROR_PREFIX,
                    err
                )
            })?;

        Ok(ret.try_into()?)
    }
}

#[async_trait]
impl ForexHistoricalRates for Api<'_> {
    async fn historical_rates(
        &self,
        date: chrono::DateTime<chrono::Utc>,
        base: iso_currency::Currency,
    ) -> crate::forex::ForexResult<crate::forex::Rates> {
        let yyyymmdd = date.format("%Y-%m-%d").to_string();
        let endpoint = CLOUDFLARE_ENDPOINT_V1
            .replace("{date}", &yyyymmdd)
            .replace("{currency_code}", base.code().to_lowercase().as_str());

        let ret: Response = self
            .client
            .get(&endpoint)
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

        Ok(ret.try_into()?)
    }
}

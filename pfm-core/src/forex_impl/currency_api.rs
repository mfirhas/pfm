// currencyapi.com
// free
// 300 reqs/month
// daily latest rates
// daily historical rates
// 10 reqs/minute

use crate::forex::{ForexHistoricalRates, Rates};
use anyhow::anyhow;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

const HISTORICAL_ENDPOINT: &str = "https://api.currencyapi.com/v3/historical";

const ERROR_PREFIX: &str = "[FOREX][currency-api]";

pub(crate) struct Api<'CLIENT> {
    key: &'static str,
    client: &'CLIENT reqwest::Client,
}

impl<'CLIENT> Api<'CLIENT> {
    pub(crate) fn new(api_key: &'static str, client: &'CLIENT reqwest::Client) -> Self {
        Self {
            key: api_key,
            client,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
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

impl TryFrom<Response> for crate::forex::Rates {
    type Error = anyhow::Error;

    fn try_from(value: Response) -> Result<Self, Self::Error> {
        let date = value
            .metadata
            .last_updated_at
            .parse::<DateTime<Utc>>()
            .map_err(|err| anyhow!("{} Failed parsing datetime: {}", ERROR_PREFIX, err))?;

        let rates = Rates {
            date,
            rates: crate::forex::Currencies {
                idr: value.rates.idr.value,
                usd: value.rates.usd.value,
                eur: value.rates.eur.value,
                gbp: value.rates.gbp.value,
                jpy: value.rates.jpy.value,
                chf: value.rates.chf.value,
                sgd: value.rates.sgd.value,
                cny: value.rates.cny.value,
                sar: value.rates.sar.value,
                xau: value.rates.xau.value,
                xag: value.rates.xag.value,
                xpt: value.rates.xpt.value,
            },
        };

        Ok(rates)
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

        let params = [
            ("apikey", self.key),
            ("base_currency", base.code()),
            ("date", yyyymmdd.as_str()),
            (
                "currencies",
                "IDR,USD,EUR,GBP,JPY,CHF,SGD,CNY,SAR,XAU,XAG,XPT",
            ),
        ];

        let ret: Response = self
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

        Ok(ret.try_into()?)
    }
}

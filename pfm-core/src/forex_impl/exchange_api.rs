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

use crate::error::AsInternalError;
use crate::forex::{
    entity::{HistoricalRates, RatesData, RatesResponse},
    interface::{ForexHistoricalRates, ForexRates},
    Currency, ForexError, ForexResult,
};
use anyhow::Context;
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
        let date = utc
            .parse::<DateTime<Utc>>()
            .context("exchange_api parse date time")
            .as_internal_err()?;
        let forex_rates = crate::forex::entity::Rates {
            latest_update: date,
            base: value.base,
            rates: RatesData {
                usd: value.api_response.rates.currencies().usd,
                cad: value.api_response.rates.currencies().cad,
                eur: value.api_response.rates.currencies().eur,
                gbp: value.api_response.rates.currencies().gbp,
                chf: value.api_response.rates.currencies().chf,
                rub: value.api_response.rates.currencies().rub,
                cny: value.api_response.rates.currencies().cny,
                jpy: value.api_response.rates.currencies().jpy,
                krw: value.api_response.rates.currencies().krw,
                hkd: value.api_response.rates.currencies().hkd,
                idr: value.api_response.rates.currencies().idr,
                myr: value.api_response.rates.currencies().myr,
                sgd: value.api_response.rates.currencies().sgd,
                thb: value.api_response.rates.currencies().thb,
                sar: value.api_response.rates.currencies().sar,
                aed: value.api_response.rates.currencies().aed,
                kwd: value.api_response.rates.currencies().kwd,
                inr: value.api_response.rates.currencies().inr,
                aud: value.api_response.rates.currencies().aud,
                nzd: value.api_response.rates.currencies().nzd,
                xau: value.api_response.rates.currencies().xau,
                xag: value.api_response.rates.currencies().xag,
                xpt: value.api_response.rates.currencies().xpt,
                btc: value.api_response.rates.currencies().btc,
                eth: value.api_response.rates.currencies().eth,
                sol: value.api_response.rates.currencies().sol,
                xrp: value.api_response.rates.currencies().xrp,
                ada: value.api_response.rates.currencies().ada,
            },
        };

        Ok(RatesResponse::new(SOURCE.into(), forex_rates))
    }
}

impl TryFrom<Response> for RatesResponse<HistoricalRates> {
    type Error = ForexError;

    fn try_from(value: Response) -> Result<Self, Self::Error> {
        let utc = format!("{}T00:00:00Z", value.api_response.date);
        let date = utc
            .parse::<DateTime<Utc>>()
            .context("exchange_api parse date time")
            .as_internal_err()?;
        let forex_rates = HistoricalRates {
            date,
            base: value.base,
            rates: RatesData {
                usd: value.api_response.rates.currencies().usd,
                cad: value.api_response.rates.currencies().cad,
                eur: value.api_response.rates.currencies().eur,
                gbp: value.api_response.rates.currencies().gbp,
                chf: value.api_response.rates.currencies().chf,
                rub: value.api_response.rates.currencies().rub,
                cny: value.api_response.rates.currencies().cny,
                jpy: value.api_response.rates.currencies().jpy,
                krw: value.api_response.rates.currencies().krw,
                hkd: value.api_response.rates.currencies().hkd,
                idr: value.api_response.rates.currencies().idr,
                myr: value.api_response.rates.currencies().myr,
                sgd: value.api_response.rates.currencies().sgd,
                thb: value.api_response.rates.currencies().thb,
                sar: value.api_response.rates.currencies().sar,
                aed: value.api_response.rates.currencies().aed,
                kwd: value.api_response.rates.currencies().kwd,
                inr: value.api_response.rates.currencies().inr,
                aud: value.api_response.rates.currencies().aud,
                nzd: value.api_response.rates.currencies().nzd,
                xau: value.api_response.rates.currencies().xau,
                xag: value.api_response.rates.currencies().xag,
                xpt: value.api_response.rates.currencies().xpt,
                btc: value.api_response.rates.currencies().btc,
                eth: value.api_response.rates.currencies().eth,
                sol: value.api_response.rates.currencies().sol,
                xrp: value.api_response.rates.currencies().xrp,
                ada: value.api_response.rates.currencies().ada,
            },
        };

        Ok(RatesResponse::new(SOURCE.into(), forex_rates))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Rates {
    #[serde(rename = "usd")]
    USD(RatesData),

    #[serde(rename = "cad")]
    CAD(RatesData),

    #[serde(rename = "eur")]
    EUR(RatesData),

    #[serde(rename = "gbp")]
    GBP(RatesData),

    #[serde(rename = "chf")]
    CHF(RatesData),

    #[serde(rename = "rub")]
    RUB(RatesData),

    #[serde(rename = "cny")]
    CNY(RatesData),

    #[serde(rename = "jpy")]
    JPY(RatesData),

    #[serde(rename = "krw")]
    KRW(RatesData),

    #[serde(rename = "hkd")]
    HKD(RatesData),

    #[serde(rename = "idr")]
    IDR(RatesData),

    #[serde(rename = "myr")]
    MYR(RatesData),

    #[serde(rename = "sgd")]
    SGD(RatesData),

    #[serde(rename = "thb")]
    THB(RatesData),

    #[serde(rename = "sar")]
    SAR(RatesData),

    #[serde(rename = "aed")]
    AED(RatesData),

    #[serde(rename = "kwd")]
    KWD(RatesData),

    #[serde(rename = "inr")]
    INR(RatesData),

    #[serde(rename = "aud")]
    AUD(RatesData),

    #[serde(rename = "nzd")]
    NZD(RatesData),

    #[serde(rename = "xau")]
    XAU(RatesData),

    #[serde(rename = "xag")]
    XAG(RatesData),

    #[serde(rename = "xpt")]
    XPT(RatesData),

    #[serde(rename = "btc")]
    BTC(RatesData),

    #[serde(rename = "eth")]
    ETH(RatesData),

    #[serde(rename = "sol")]
    SOL(RatesData),

    #[serde(rename = "xrp")]
    XRP(RatesData),

    #[serde(rename = "ada")]
    ADA(RatesData),
}

impl Rates {
    pub fn currencies(&self) -> &RatesData {
        match self {
            Rates::USD(currencies) => currencies,
            Rates::CAD(currencies) => currencies,
            Rates::EUR(currencies) => currencies,
            Rates::GBP(currencies) => currencies,
            Rates::CHF(currencies) => currencies,
            Rates::RUB(currencies) => currencies,
            Rates::CNY(currencies) => currencies,
            Rates::JPY(currencies) => currencies,
            Rates::KRW(currencies) => currencies,
            Rates::HKD(currencies) => currencies,
            Rates::IDR(currencies) => currencies,
            Rates::MYR(currencies) => currencies,
            Rates::SGD(currencies) => currencies,
            Rates::THB(currencies) => currencies,
            Rates::SAR(currencies) => currencies,
            Rates::AED(currencies) => currencies,
            Rates::KWD(currencies) => currencies,
            Rates::INR(currencies) => currencies,
            Rates::AUD(currencies) => currencies,
            Rates::NZD(currencies) => currencies,
            Rates::XAU(currencies) => currencies,
            Rates::XAG(currencies) => currencies,
            Rates::XPT(currencies) => currencies,
            Rates::BTC(currencies) => currencies,
            Rates::ETH(currencies) => currencies,
            Rates::SOL(currencies) => currencies,
            Rates::XRP(currencies) => currencies,
            Rates::ADA(currencies) => currencies,
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
            .context("exchange_api invoking latest rates api")
            .as_internal_err()?
            .error_for_status()
            .context("non 200/201 error")
            .as_internal_err()?
            .json()
            .await
            .context("exchange api parsing into json")
            .as_internal_err()?;

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
            .context("exchange_api invoking historical rates api")
            .as_internal_err()?
            .error_for_status()
            .context("exchange_api non 200/201 error")
            .as_internal_err()?
            .json()
            .await
            .context("exchange_api converting response to json")
            .as_internal_err()?;

        let ret = Response {
            base,
            api_response: ret,
        };

        Ok(ret.try_into()?)
    }
}

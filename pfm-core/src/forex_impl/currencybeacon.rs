use std::{collections::HashMap, str::FromStr};

use anyhow::{anyhow, Context};
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::forex::{
    entity::{HistoricalRates, Rates, RatesData, RatesResponse},
    interface::{AsInternalError, ForexHistoricalRates, ForexRates, ForexTimeseriesRates},
    Currency, ForexError, ForexResult,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/**
https://currencybeacon.com
5,000 API Requests
Hourly updates
Latest Rates
Historical Rates
Rate limit: // TODO ask currencybeacon for rate limit
"End-of-day rates are available historically for all days going back to 1st January, 1996."
*/

const LATEST_ENDPOINT: &str = "https://api.currencybeacon.com/v1/latest";
const HISTORICAL_ENDPOINT: &str = "https://api.currencybeacon.com/v1/historical";
const TIMESERIES_ENDPOINT: &str = "https://api.currencybeacon.com/v1/timeseries";
const SOURCE: &str = "currencybeacon.com";
const END_OF_DAY_HOUR: &str = "T23:59:59Z";

#[derive(Clone)]
pub struct Api {
    key: &'static str,
    client: reqwest::Client,
}

impl Api {
    pub fn new(key: &'static str, http_client: reqwest::Client) -> Self {
        Self {
            key,
            client: http_client,
        }
    }
}

impl TryFrom<Response> for RatesResponse<Rates> {
    type Error = ForexError;

    fn try_from(value: Response) -> Result<Self, Self::Error> {
        let date = value
            .response
            .date
            .parse::<DateTime<Utc>>()
            .context("currencybeacon parse latest rates datetime")
            .as_internal_err()?;

        let base = Currency::from_str(&value.response.base.as_str())
            .context("currencybeacon parse base currency")
            .as_internal_err()?;

        let rates = Rates {
            latest_update: date,
            base,
            rates: RatesData {
                usd: value.response.rates.usd.unwrap_or_default(),
                cad: value.response.rates.cad.unwrap_or_default(),
                eur: value.response.rates.eur.unwrap_or_default(),
                gbp: value.response.rates.gbp.unwrap_or_default(),
                chf: value.response.rates.chf.unwrap_or_default(),
                rub: value.response.rates.rub.unwrap_or_default(),
                cny: value.response.rates.cny.unwrap_or_default(),
                jpy: value.response.rates.jpy.unwrap_or_default(),
                krw: value.response.rates.krw.unwrap_or_default(),
                hkd: value.response.rates.hkd.unwrap_or_default(),
                idr: value.response.rates.idr.unwrap_or_default(),
                myr: value.response.rates.myr.unwrap_or_default(),
                sgd: value.response.rates.sgd.unwrap_or_default(),
                thb: value.response.rates.thb.unwrap_or_default(),
                sar: value.response.rates.sar.unwrap_or_default(),
                aed: value.response.rates.aed.unwrap_or_default(),
                kwd: value.response.rates.kwd.unwrap_or_default(),
                inr: value.response.rates.inr.unwrap_or_default(),
                aud: value.response.rates.aud.unwrap_or_default(),
                nzd: value.response.rates.nzd.unwrap_or_default(),
                xau: value.response.rates.xau.unwrap_or_default(),
                xag: value.response.rates.xag.unwrap_or_default(),
                xpt: value.response.rates.xpt.unwrap_or_default(),
                btc: value.response.rates.btc.unwrap_or_default(),
                eth: value.response.rates.eth.unwrap_or_default(),
                sol: value.response.rates.sol.unwrap_or_default(),
                xrp: value.response.rates.xrp.unwrap_or_default(),
                ada: value.response.rates.ada.unwrap_or_default(),
            },
        };

        Ok(RatesResponse::new(SOURCE.into(), rates))
    }
}

impl TryFrom<Response> for RatesResponse<HistoricalRates> {
    type Error = ForexError;

    fn try_from(value: Response) -> Result<Self, Self::Error> {
        let date_time_str = format!("{}{}", value.response.date, END_OF_DAY_HOUR);
        let date = date_time_str
            .parse::<DateTime<Utc>>()
            .context("currencybeacon parse historical rates datetime")
            .as_internal_err()?;

        let base = Currency::from_str(&value.response.base.as_str())
            .context("currencybeacon parse base currency")
            .as_internal_err()?;

        let historical_rates = HistoricalRates {
            date,
            base,
            rates: RatesData {
                usd: value.response.rates.usd.unwrap_or_default(),
                cad: value.response.rates.cad.unwrap_or_default(),
                eur: value.response.rates.eur.unwrap_or_default(),
                gbp: value.response.rates.gbp.unwrap_or_default(),
                chf: value.response.rates.chf.unwrap_or_default(),
                rub: value.response.rates.rub.unwrap_or_default(),
                cny: value.response.rates.cny.unwrap_or_default(),
                jpy: value.response.rates.jpy.unwrap_or_default(),
                krw: value.response.rates.krw.unwrap_or_default(),
                hkd: value.response.rates.hkd.unwrap_or_default(),
                idr: value.response.rates.idr.unwrap_or_default(),
                myr: value.response.rates.myr.unwrap_or_default(),
                sgd: value.response.rates.sgd.unwrap_or_default(),
                thb: value.response.rates.thb.unwrap_or_default(),
                sar: value.response.rates.sar.unwrap_or_default(),
                aed: value.response.rates.aed.unwrap_or_default(),
                kwd: value.response.rates.kwd.unwrap_or_default(),
                inr: value.response.rates.inr.unwrap_or_default(),
                aud: value.response.rates.aud.unwrap_or_default(),
                nzd: value.response.rates.nzd.unwrap_or_default(),
                xau: value.response.rates.xau.unwrap_or_default(),
                xag: value.response.rates.xag.unwrap_or_default(),
                xpt: value.response.rates.xpt.unwrap_or_default(),
                btc: value.response.rates.btc.unwrap_or_default(),
                eth: value.response.rates.eth.unwrap_or_default(),
                sol: value.response.rates.sol.unwrap_or_default(),
                xrp: value.response.rates.xrp.unwrap_or_default(),
                ada: value.response.rates.ada.unwrap_or_default(),
            },
        };

        Ok(RatesResponse::new(SOURCE.into(), historical_rates))
    }
}

// (Currency, ...), Currency is base currency
impl TryFrom<(Currency, TimeseriesResponse)> for RatesResponse<Vec<HistoricalRates>> {
    type Error = ForexError;

    fn try_from(value: (Currency, TimeseriesResponse)) -> Result<Self, Self::Error> {
        let mut historical_rates_list: Vec<HistoricalRates> = vec![];
        for (date_str, r) in value.1.response {
            let date_time_str = format!("{}{}", date_str, END_OF_DAY_HOUR);
            let date = date_time_str
                .parse::<DateTime<Utc>>()
                .context("currencybeacon parse historical rates datetime")
                .as_internal_err()?;
            let historical_rates = HistoricalRates {
                date,
                base: value.0,
                rates: RatesData {
                    usd: r.usd.unwrap_or_default(),
                    cad: r.cad.unwrap_or_default(),
                    eur: r.eur.unwrap_or_default(),
                    gbp: r.gbp.unwrap_or_default(),
                    chf: r.chf.unwrap_or_default(),
                    rub: r.rub.unwrap_or_default(),
                    cny: r.cny.unwrap_or_default(),
                    jpy: r.jpy.unwrap_or_default(),
                    krw: r.krw.unwrap_or_default(),
                    hkd: r.hkd.unwrap_or_default(),
                    idr: r.idr.unwrap_or_default(),
                    myr: r.myr.unwrap_or_default(),
                    sgd: r.sgd.unwrap_or_default(),
                    thb: r.thb.unwrap_or_default(),
                    sar: r.sar.unwrap_or_default(),
                    aed: r.aed.unwrap_or_default(),
                    kwd: r.kwd.unwrap_or_default(),
                    inr: r.inr.unwrap_or_default(),
                    aud: r.aud.unwrap_or_default(),
                    nzd: r.nzd.unwrap_or_default(),
                    xau: r.xau.unwrap_or_default(),
                    xag: r.xag.unwrap_or_default(),
                    xpt: r.xpt.unwrap_or_default(),
                    btc: r.btc.unwrap_or_default(),
                    eth: r.eth.unwrap_or_default(),
                    sol: r.sol.unwrap_or_default(),
                    xrp: r.xrp.unwrap_or_default(),
                    ada: r.ada.unwrap_or_default(),
                },
            };

            historical_rates_list.push(historical_rates);
        }

        // sort ASC
        historical_rates_list.sort_by_key(|rates| rates.date);

        Ok(RatesResponse::new(SOURCE.into(), historical_rates_list))
    }
}

// --- latest and historical rates response
#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub meta: Meta,
    pub response: ResponseData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Meta {
    pub code: u16,
    pub disclaimer: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseData {
    pub date: String,
    pub base: String,
    pub rates: ResponseRates,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseRates {
    #[serde(rename = "USD", default)]
    pub usd: Option<Decimal>,
    #[serde(rename = "CAD", default)]
    pub cad: Option<Decimal>,
    #[serde(rename = "EUR", default)]
    pub eur: Option<Decimal>,
    #[serde(rename = "GBP", default)]
    pub gbp: Option<Decimal>,
    #[serde(rename = "CHF", default)]
    pub chf: Option<Decimal>,
    #[serde(rename = "RUB", default)]
    pub rub: Option<Decimal>,
    #[serde(rename = "CNY", default)]
    pub cny: Option<Decimal>,
    #[serde(rename = "JPY", default)]
    pub jpy: Option<Decimal>,
    #[serde(rename = "KRW", default)]
    pub krw: Option<Decimal>,
    #[serde(rename = "HKD", default)]
    pub hkd: Option<Decimal>,
    #[serde(rename = "IDR", default)]
    pub idr: Option<Decimal>,
    #[serde(rename = "MYR", default)]
    pub myr: Option<Decimal>,
    #[serde(rename = "SGD", default)]
    pub sgd: Option<Decimal>,
    #[serde(rename = "THB", default)]
    pub thb: Option<Decimal>,
    #[serde(rename = "SAR", default)]
    pub sar: Option<Decimal>,
    #[serde(rename = "AED", default)]
    pub aed: Option<Decimal>,
    #[serde(rename = "KWD", default)]
    pub kwd: Option<Decimal>,
    #[serde(rename = "INR", default)]
    pub inr: Option<Decimal>,
    #[serde(rename = "AUD", default)]
    pub aud: Option<Decimal>,
    #[serde(rename = "NZD", default)]
    pub nzd: Option<Decimal>,
    #[serde(rename = "XAU", default)]
    pub xau: Option<Decimal>,
    #[serde(rename = "XAG", default)]
    pub xag: Option<Decimal>,
    #[serde(rename = "XPT", default)]
    pub xpt: Option<Decimal>,
    #[serde(rename = "BTC", default)]
    pub btc: Option<Decimal>,
    #[serde(rename = "ETH", default)]
    pub eth: Option<Decimal>,
    #[serde(rename = "SOL", default)]
    pub sol: Option<Decimal>,
    #[serde(rename = "XRP", default)]
    pub xrp: Option<Decimal>,
    #[serde(rename = "ADA", default)]
    pub ada: Option<Decimal>,
}

// --- END

// --- timeseries dto
#[derive(Serialize, Deserialize, Debug)]
pub struct TimeseriesResponse {
    #[serde(rename = "meta")]
    pub meta: Meta,

    /// object of date(YYYY-MM-DD) to its rates
    #[serde(rename = "response")]
    pub response: HashMap<String, ExchangeRates>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExchangeRates {
    #[serde(rename = "USD")]
    pub usd: Option<Decimal>,

    #[serde(rename = "CAD")]
    pub cad: Option<Decimal>,

    #[serde(rename = "EUR")]
    pub eur: Option<Decimal>,

    #[serde(rename = "GBP")]
    pub gbp: Option<Decimal>,

    #[serde(rename = "CHF")]
    pub chf: Option<Decimal>,

    #[serde(rename = "RUB")]
    pub rub: Option<Decimal>,

    #[serde(rename = "CNY")]
    pub cny: Option<Decimal>,

    #[serde(rename = "JPY")]
    pub jpy: Option<Decimal>,

    #[serde(rename = "KRW")]
    pub krw: Option<Decimal>,

    #[serde(rename = "HKD")]
    pub hkd: Option<Decimal>,

    #[serde(rename = "IDR")]
    pub idr: Option<Decimal>,

    #[serde(rename = "MYR")]
    pub myr: Option<Decimal>,

    #[serde(rename = "SGD")]
    pub sgd: Option<Decimal>,

    #[serde(rename = "THB")]
    pub thb: Option<Decimal>,

    #[serde(rename = "SAR")]
    pub sar: Option<Decimal>,

    #[serde(rename = "AED")]
    pub aed: Option<Decimal>,

    #[serde(rename = "KWD")]
    pub kwd: Option<Decimal>,

    #[serde(rename = "INR")]
    pub inr: Option<Decimal>,

    #[serde(rename = "AUD")]
    pub aud: Option<Decimal>,

    #[serde(rename = "NZD")]
    pub nzd: Option<Decimal>,

    // Precious metals
    #[serde(rename = "XAU")]
    pub xau: Option<Decimal>,

    #[serde(rename = "XAG")]
    pub xag: Option<Decimal>,

    #[serde(rename = "XPT")]
    pub xpt: Option<Decimal>,

    // Crypto
    #[serde(rename = "BTC")]
    pub btc: Option<Decimal>,

    #[serde(rename = "ETH")]
    pub eth: Option<Decimal>,

    #[serde(rename = "SOL")]
    pub sol: Option<Decimal>,

    #[serde(rename = "XRP")]
    pub xrp: Option<Decimal>,

    #[serde(rename = "ADA")]
    pub ada: Option<Decimal>,
}
// --- END

#[async_trait]
impl ForexRates for Api {
    async fn rates(&self, base: Currency) -> ForexResult<RatesResponse<Rates>> {
        let symbols = Currency::to_comma_separated_list_str();
        let params = [
            ("api_key", self.key),
            ("base", base.code()),
            ("symbols", symbols.as_str()),
        ];

        let ret_str = self
            .client
            .get(LATEST_ENDPOINT)
            .query(&params)
            .send()
            .await
            .context("currencybeacon invoking latest api")
            .as_internal_err()?
            .text()
            .await
            .context("currencybeacon fetching latest resp in text")
            .as_internal_err()?;

        let resp = serde_json::from_str::<Response>(&ret_str)
            .map_err(|err| {
                anyhow!(
                    "currencybeacon failed parsing latest into JSON: {}, {}",
                    &ret_str,
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
        date: DateTime<Utc>,
        base: Currency,
    ) -> ForexResult<RatesResponse<HistoricalRates>> {
        let symbols = Currency::to_comma_separated_list_str();
        let yyyymmdd = date.format("%Y-%m-%d").to_string();
        let params = [
            ("api_key", self.key),
            ("base", base.code()),
            ("date", yyyymmdd.as_str()),
            ("symbols", symbols.as_str()),
        ];

        let ret_str = self
            .client
            .get(HISTORICAL_ENDPOINT)
            .query(&params)
            .send()
            .await
            .context("currencybeacon invoking historical api")
            .as_internal_err()?
            .text()
            .await
            .context("currencybeacon fetching historical resp in text")
            .as_internal_err()?;

        let resp = serde_json::from_str::<Response>(&ret_str)
            .map_err(|err| {
                anyhow!(
                    "currencybeacon failed parsing historical into JSON: {}, {}",
                    &ret_str,
                    err
                )
            })
            .as_internal_err()?;

        Ok(resp.try_into()?)
    }
}

#[async_trait]
impl ForexTimeseriesRates for Api {
    async fn timeseries_rates(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        base: Currency,
    ) -> ForexResult<RatesResponse<Vec<HistoricalRates>>> {
        let symbols = Currency::to_comma_separated_list_str();
        let from = start_date.format("%Y-%m-%d").to_string();
        let to = end_date.format("%Y-%m-%d").to_string();

        let params = [
            ("api_key", self.key),
            ("base", base.code()),
            ("start_date", from.as_str()),
            ("end_date", to.as_str()),
            ("symbols", symbols.as_str()),
        ];

        let ret_str = self
            .client
            .get(TIMESERIES_ENDPOINT)
            .query(&params)
            .send()
            .await
            .context("currencybeacon invoking timeseries api")
            .as_internal_err()?
            .text()
            .await
            .context("currencybeacon fetching timeseries resp in text")
            .as_internal_err()?;

        let resp = serde_json::from_str::<TimeseriesResponse>(&ret_str)
            .map_err(|err| {
                anyhow!(
                    "currencybeacon failed parsing timeseries into JSON: {}, {}",
                    &ret_str,
                    err
                )
            })
            .as_internal_err()?;

        let resp = (base, resp);

        Ok(resp.try_into()?)
    }
}

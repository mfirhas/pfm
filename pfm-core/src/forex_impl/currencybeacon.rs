use std::{collections::HashMap, str::FromStr};

use anyhow::{Context, anyhow};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal_macros::dec;

use crate::error::AsInternalError;
use crate::{
    forex::{
        Currency, ForexError, ForexResult,
        entity::{Rates, RatesData, RatesResponse},
        interface::{ForexHistoricalRates, ForexRates, ForexTimeseriesRates},
    },
    global::{self},
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
"The timeseries endpoint only supports a 7 year range at a time, please split your query into multie ranges"
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

    /// currencybeacon doesn't provide price for Solana, so fetch it from other source instead.

    /// fetch from twelvedata.com /exchange_rate
    async fn latest_solana(&self, base: Currency) -> ForexResult<Decimal> {
        const TWELVEDATA_LATEST_ENDPOINT: &str = "https://api.twelvedata.com/exchange_rate";
        let api_key = &global::config().forex_twelvedata_api_key;
        let symbol = format!("{}/SOL", base);
        let params = [("apikey", api_key.as_str()), ("symbol", &symbol)];

        #[derive(Debug, Serialize, Deserialize)]
        struct SolanaResponse {
            symbol: String,
            rate: Decimal,
            timestamp: i64,
        }

        let ret_text = global::http_client()
            .get(TWELVEDATA_LATEST_ENDPOINT)
            .query(&params)
            .send()
            .await
            .context("currencybeacon twelvedata latest solana invoking api")
            .as_internal_err()?
            .text()
            .await
            .context("currencybeacon twelvedata latest solana string response")
            .as_internal_err()?;

        let ret: SolanaResponse = serde_json::from_str(&ret_text)
            .map_err(|err| {
                anyhow!(
                    "currencybeacon twelvedata parsing latest json: {}, err: {}",
                    &ret_text,
                    err
                )
            })
            .as_internal_err()?;

        Ok(ret.rate)
    }

    /// fetch from twelvedata.com /time_series
    async fn historical_solana(&self, base: Currency, date: DateTime<Utc>) -> ForexResult<Decimal> {
        const TWELVEDATA_TIMESERIES_ENDPOINT: &str = "https://api.twelvedata.com/time_series";
        let api_key = &global::config().forex_twelvedata_api_key;
        // symbol for time_series endpoint doesn't provide USD/BTC, use this instead and calculate from it.
        let symbol = format!("SOL/{}", base);
        let date = date.format("%Y-%m-%d").to_string();
        let params = [
            ("apikey", api_key.as_str()),
            ("symbol", &symbol),
            ("interval", "1day"),
            ("date", date.as_str()),
        ];

        #[derive(Debug, Serialize, Deserialize)]
        struct SolanaTimeseriesResponse {
            values: Vec<PriceData>,
        }

        #[derive(Debug, Serialize, Deserialize)]
        struct PriceData {
            datetime: String,
            open: Decimal,
            high: Decimal,
            low: Decimal,
            close: Decimal,
        }

        let ret_text = global::http_client()
            .get(TWELVEDATA_TIMESERIES_ENDPOINT)
            .query(&params)
            .send()
            .await
            .context("currencybeacon twelvedata historical solana invoking api")
            .as_internal_err()?
            .text()
            .await
            .context("currencybeacon twelvedata historical solana string response")
            .as_internal_err()?;

        let ret: SolanaTimeseriesResponse = serde_json::from_str(&ret_text)
            .map_err(|err| {
                anyhow!(
                    "currencybeacon twelvedata parsing historical json: {}, err: {}",
                    &ret_text,
                    err
                )
            })
            .as_internal_err()?;

        if ret.values.is_empty() {
            return Err(ForexError::internal_error(
                "currencybeacon twelvedata historical returned price is empty",
            ));
        } else {
            let price_data = &ret.values[0];
            if &price_data.datetime != &date {
                return Err(ForexError::internal_error(&format!(
                    "currencybeacon twelvedata historical returned mismatch date, expected: {}, got: {}",
                    &date, &price_data.datetime
                )));
            }

            // rate is for SOL/base, so to get 1 base = X SOL, divide 1 with SOL price
            let usd_sol = dec!(1).checked_div(price_data.close).unwrap_or_default();

            Ok(usd_sol)
        }
    }
}

#[cfg(test)]
mod api_tests {
    use chrono::{TimeZone, Utc};

    use crate::{forex::Currency, global};

    #[tokio::test]
    #[ignore]
    async fn test_solana_latest_price() {
        let base = Currency::USD;
        let api = super::Api {
            key: &global::config().forex_twelvedata_api_key,
            client: global::http_client(),
        };

        let ret = api.latest_solana(base).await;
        dbg!(&ret);
    }

    #[tokio::test]
    #[ignore]
    async fn test_solana_historical_price() {
        let base = Currency::USD;
        let api = super::Api {
            key: &global::config().forex_twelvedata_api_key,
            client: global::http_client(),
        };
        let date = Utc.with_ymd_and_hms(2022, 8, 1, 0, 0, 0).unwrap();

        let ret = api.historical_solana(base, date).await;
        dbg!(&ret);
    }
}

impl TryFrom<(Response, Decimal)> for RatesResponse<Rates> {
    type Error = ForexError;

    fn try_from(
        (value, twelvedata_solana_price): (Response, Decimal),
    ) -> Result<Self, Self::Error> {
        let date = if let Ok(date_time) = value.response.date.parse::<DateTime<Utc>>() {
            date_time
        } else {
            format!("{}{}", value.response.date, END_OF_DAY_HOUR)
                .parse::<DateTime<Utc>>()
                .context("currencybeacon parse historical rates datetime")
                .as_internal_err()?
        };

        let base = Currency::from_str(&value.response.base.as_str())
            .context("currencybeacon parse base currency")
            .as_internal_err()?;

        // check solana price if exist, else use twelvedata price
        let solana_price = match value.response.rates.sol {
            Some(currencybeacon_solana_price) => {
                if currencybeacon_solana_price.is_zero() {
                    twelvedata_solana_price
                } else {
                    currencybeacon_solana_price
                }
            }
            None => twelvedata_solana_price,
        };

        let rates = Rates {
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
                sol: solana_price,
                xrp: value.response.rates.xrp.unwrap_or_default(),
                ada: value.response.rates.ada.unwrap_or_default(),
            },
        };

        Ok(RatesResponse::new(SOURCE.into(), rates))
    }
}

struct RatesResponseList(Vec<RatesResponse<Rates>>);

// (Currency, ...), Currency is base currency
impl TryFrom<(Currency, TimeseriesResponse)> for RatesResponseList {
    type Error = ForexError;

    fn try_from(value: (Currency, TimeseriesResponse)) -> Result<Self, Self::Error> {
        let mut rates_response_list: Vec<RatesResponse<Rates>> = vec![];
        for (date_str, r) in value.1.response {
            let date_time_str = format!("{}{}", date_str, END_OF_DAY_HOUR);
            let date = date_time_str
                .parse::<DateTime<Utc>>()
                .context("currencybeacon parse historical rates datetime")
                .as_internal_err()?;
            let historical_rates = Rates {
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

            let rates_response = RatesResponse::new(SOURCE.into(), historical_rates);

            rates_response_list.push(rates_response);
        }

        // sort ASC
        rates_response_list.sort_by_key(|rates| rates.data.date);

        Ok(RatesResponseList(rates_response_list))
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

        // solana price
        let solana_price = self.latest_solana(base).await.unwrap_or_default();
        let resp = (resp, solana_price);

        Ok(resp.try_into()?)
    }
}

#[async_trait]
impl ForexHistoricalRates for Api {
    async fn historical_rates(
        &self,
        date: DateTime<Utc>,
        base: Currency,
    ) -> ForexResult<RatesResponse<Rates>> {
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

        // solana price
        let solana_price = self.historical_solana(base, date).await.unwrap_or_default();
        let resp = (resp, solana_price);

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
    ) -> ForexResult<Vec<RatesResponse<Rates>>> {
        if start_date > end_date {
            return Err(ForexError::client_error(
                "start date cannot be bigger than end date",
            ));
        }

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

        let resp = RatesResponseList::try_from(resp)?.0;

        Ok(resp)
    }
}

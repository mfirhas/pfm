use anyhow::{anyhow, Context};
use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::forex::{
    entity::{HistoricalRates, Rates, RatesData, RatesResponse},
    interface::{AsInternalError, ForexHistoricalRates, ForexRates},
    Currency, ForexError, ForexResult,
};

const LATEST_ENDPOINT: &str = "https://marketdata.tradermade.com/api/v1/live";
const HISTORICAL_ENDPOINT: &str = "https://marketdata.tradermade.com/api/v1/historical";
const SOURCE: &str = "tradermade.com";

// https://tradermade.com/
/**
1,000 Requests
Personal License
Live Tick Rates
Historical Rates
Historical Minute
*/

// latest rates dto
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct LatestResponse {
    endpoint: String,
    quotes: Vec<Quote>,
    requested_time: String,
    timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct Quote {
    ask: Decimal,
    base_currency: String,
    bid: Decimal,
    mid: Decimal,
    quote_currency: String,
}
// END

// historical rates dto

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct HistoricalResponse {
    date: String,
    endpoint: String,
    quotes: Vec<QuoteEnum>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
enum QuoteEnum {
    Data(QuoteData),
    Error(QuoteError),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct QuoteData {
    base_currency: String,
    quote_currency: String,
    open: Decimal,
    high: Decimal,
    low: Decimal,
    close: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct QuoteError {
    error: u16,
    instrument: String,
    message: String,
}
// END

impl RatesData {
    pub fn set_base(base: Currency) -> RatesData {
        let mut ret = RatesData::default();
        match base {
            Currency::USD => ret.usd = dec!(1),
            Currency::CAD => ret.cad = dec!(1),
            Currency::EUR => ret.eur = dec!(1),
            Currency::GBP => ret.gbp = dec!(1),
            Currency::CHF => ret.chf = dec!(1),
            Currency::RUB => ret.rub = dec!(1),
            Currency::CNY => ret.cny = dec!(1),
            Currency::JPY => ret.jpy = dec!(1),
            Currency::KRW => ret.krw = dec!(1),
            Currency::HKD => ret.hkd = dec!(1),
            Currency::IDR => ret.idr = dec!(1),
            Currency::MYR => ret.myr = dec!(1),
            Currency::SGD => ret.sgd = dec!(1),
            Currency::THB => ret.thb = dec!(1),
            Currency::SAR => ret.sar = dec!(1),
            Currency::AED => ret.aed = dec!(1),
            Currency::KWD => ret.kwd = dec!(1),
            Currency::INR => ret.inr = dec!(1),
            Currency::AUD => ret.aud = dec!(1),
            Currency::NZD => ret.nzd = dec!(1),
            Currency::XAU => ret.xau = dec!(1),
            Currency::XAG => ret.xag = dec!(1),
            Currency::XPT => ret.xpt = dec!(1),
            Currency::BTC => ret.btc = dec!(1),
            Currency::ETH => ret.eth = dec!(1),
            Currency::SOL => ret.sol = dec!(1),
            Currency::XRP => ret.xrp = dec!(1),
            Currency::ADA => ret.ada = dec!(1),
        }
        ret
    }
}

impl TryFrom<(Currency, LatestResponse)> for RatesResponse<Rates> {
    type Error = ForexError;

    fn try_from(value: (Currency, LatestResponse)) -> Result<Self, Self::Error> {
        if value.1.quotes.len() != Currency::currencies_count() - 1 {
            return Err(ForexError::internal_error(
                format!(
                    "tradermade mismatch api response number of quotes, expected {}, got {}",
                    Currency::currencies_count(),
                    value.1.quotes.len()
                )
                .as_str(),
            ));
        }
        let date = Utc
            .timestamp_opt(value.1.timestamp as i64, 0)
            .single()
            .ok_or(ForexError::internal_error(
                "tradermade converting latest rates unix epoch to utc",
            ))?;

        let mut rates = Rates {
            latest_update: date,
            base: value.0,
            rates: RatesData::set_base(value.0),
        };
        for rate in value.1.quotes {
            {
                let base = rate
                    .base_currency
                    .parse::<Currency>()
                    .context("tradermade latest into rates response")
                    .as_internal_err()?;
                if base != value.0 {
                    return Err(ForexError::internal_error(
                        "tradermade mismatch latest base currency",
                    ));
                }
            };

            let target_curr = rate
                .quote_currency
                .parse::<Currency>()
                .context("tradermade latest quoted currency parsing")
                .as_internal_err()?;
            if target_curr == value.0 {
                return Err(ForexError::internal_error(
                    "tradermade latest there should be no base in quote currency",
                ));
            }

            match target_curr {
                Currency::USD => rates.rates.usd = rate.mid,
                Currency::CAD => rates.rates.cad = rate.mid,
                Currency::EUR => rates.rates.eur = rate.mid,
                Currency::GBP => rates.rates.gbp = rate.mid,
                Currency::CHF => rates.rates.chf = rate.mid,
                Currency::RUB => rates.rates.rub = rate.mid,
                Currency::CNY => rates.rates.cny = rate.mid,
                Currency::JPY => rates.rates.jpy = rate.mid,
                Currency::KRW => rates.rates.krw = rate.mid,
                Currency::HKD => rates.rates.hkd = rate.mid,
                Currency::IDR => rates.rates.idr = rate.mid,
                Currency::MYR => rates.rates.myr = rate.mid,
                Currency::SGD => rates.rates.sgd = rate.mid,
                Currency::THB => rates.rates.thb = rate.mid,
                Currency::SAR => rates.rates.sar = rate.mid,
                Currency::AED => rates.rates.aed = rate.mid,
                Currency::KWD => rates.rates.kwd = rate.mid,
                Currency::INR => rates.rates.inr = rate.mid,
                Currency::AUD => rates.rates.aud = rate.mid,
                Currency::NZD => rates.rates.nzd = rate.mid,
                Currency::XAU => rates.rates.xau = rate.mid,
                Currency::XAG => rates.rates.xag = rate.mid,
                Currency::XPT => rates.rates.xpt = rate.mid,
                Currency::BTC => rates.rates.btc = rate.mid,
                Currency::ETH => rates.rates.eth = rate.mid,
                Currency::SOL => rates.rates.sol = rate.mid,
                Currency::XRP => rates.rates.xrp = rate.mid,
                Currency::ADA => rates.rates.ada = rate.mid,
            }
        }

        Ok(RatesResponse::new(SOURCE.into(), rates))
    }
}

impl TryFrom<(Currency, HistoricalResponse)> for RatesResponse<HistoricalRates> {
    type Error = ForexError;

    fn try_from(value: (Currency, HistoricalResponse)) -> Result<Self, Self::Error> {
        if value.1.quotes.len() != Currency::currencies_count() - 1 {
            return Err(ForexError::internal_error(
                "tradermade historical incorrect quotes count",
            ));
        }
        let date_str = format!("{}T23:59:59Z", value.1.date);
        let date = date_str
            .parse::<DateTime<Utc>>()
            .context("tradermade historical parse date to rates response")
            .as_internal_err()?;

        let mut historical_rates = HistoricalRates {
            date,
            base: value.0,
            rates: RatesData::set_base(value.0),
        };
        for rate in value.1.quotes {
            match rate {
                QuoteEnum::Error(_) => continue,
                QuoteEnum::Data(rate) => {
                    {
                        let base = rate
                            .base_currency
                            .parse::<Currency>()
                            .context("tradermade latest into rates response")
                            .as_internal_err()?;
                        if base != value.0 {
                            return Err(ForexError::internal_error(
                                "tradermade mismatch latest base currency",
                            ));
                        }
                    };

                    let target_curr = rate
                        .quote_currency
                        .parse::<Currency>()
                        .context("tradermade latest quoted currency parsing")
                        .as_internal_err()?;
                    if target_curr == value.0 {
                        return Err(ForexError::internal_error(
                            "tradermade latest there should be no base in quote currency",
                        ));
                    }

                    match target_curr {
                        Currency::USD => historical_rates.rates.usd = rate.close,
                        Currency::CAD => historical_rates.rates.cad = rate.close,
                        Currency::EUR => historical_rates.rates.eur = rate.close,
                        Currency::GBP => historical_rates.rates.gbp = rate.close,
                        Currency::CHF => historical_rates.rates.chf = rate.close,
                        Currency::RUB => historical_rates.rates.rub = rate.close,
                        Currency::CNY => historical_rates.rates.cny = rate.close,
                        Currency::JPY => historical_rates.rates.jpy = rate.close,
                        Currency::KRW => historical_rates.rates.krw = rate.close,
                        Currency::HKD => historical_rates.rates.hkd = rate.close,
                        Currency::IDR => historical_rates.rates.idr = rate.close,
                        Currency::MYR => historical_rates.rates.myr = rate.close,
                        Currency::SGD => historical_rates.rates.sgd = rate.close,
                        Currency::THB => historical_rates.rates.thb = rate.close,
                        Currency::SAR => historical_rates.rates.sar = rate.close,
                        Currency::AED => historical_rates.rates.aed = rate.close,
                        Currency::KWD => historical_rates.rates.kwd = rate.close,
                        Currency::INR => historical_rates.rates.inr = rate.close,
                        Currency::AUD => historical_rates.rates.aud = rate.close,
                        Currency::NZD => historical_rates.rates.nzd = rate.close,
                        Currency::XAU => historical_rates.rates.xau = rate.close,
                        Currency::XAG => historical_rates.rates.xag = rate.close,
                        Currency::XPT => historical_rates.rates.xpt = rate.close,
                        Currency::BTC => historical_rates.rates.btc = rate.close,
                        Currency::ETH => historical_rates.rates.eth = rate.close,
                        Currency::SOL => historical_rates.rates.sol = rate.close,
                        Currency::XRP => historical_rates.rates.xrp = rate.close,
                        Currency::ADA => historical_rates.rates.ada = rate.close,
                    }
                }
            }
        }

        Ok(RatesResponse::new(SOURCE.into(), historical_rates))
    }
}

#[derive(Clone)]
pub struct Api {
    api_key: &'static str,
    client: reqwest::Client,
}

impl Api {
    pub fn new(api_key: &'static str, client: reqwest::Client) -> Self {
        Self { api_key, client }
    }
}

#[async_trait]
impl ForexRates for Api {
    async fn rates(&self, base: Currency) -> ForexResult<RatesResponse<Rates>> {
        let currencies = Currency::to_comma_separated_pair_list_str(base);

        let params = [("api_key", self.api_key), ("currency", currencies.as_str())];

        let resp_str = self
            .client
            .get(LATEST_ENDPOINT)
            .query(&params)
            .send()
            .await
            .context("tradermade invoking latest api")
            .as_internal_err()?
            .text()
            .await
            .context("tradermade fetch latest resp as text")
            .as_internal_err()?;

        let ret = serde_json::from_str::<LatestResponse>(&resp_str)
            .map_err(|err| {
                anyhow!(
                    "tradermade parsing latest resp to json: {}, err: {}",
                    &resp_str,
                    err
                )
            })
            .as_internal_err()?;

        let ret = (base, ret);

        Ok(ret.try_into()?)
    }
}

#[async_trait]
impl ForexHistoricalRates for Api {
    async fn historical_rates(
        &self,
        date: DateTime<Utc>,
        base: Currency,
    ) -> ForexResult<RatesResponse<HistoricalRates>> {
        let currencies = Currency::to_comma_separated_pair_list_str(base);
        let date = date.format("%Y-%m-%d").to_string();

        let params = [
            ("api_key", self.api_key),
            ("currency", currencies.as_str()),
            ("date", date.as_str()),
        ];

        let resp_str = self
            .client
            .get(HISTORICAL_ENDPOINT)
            .query(&params)
            .send()
            .await
            .context("tradermade invoking historical api")
            .as_internal_err()?
            .text()
            .await
            .context("tradermade fetch historical resp as text")
            .as_internal_err()?;

        let ret = serde_json::from_str::<HistoricalResponse>(&resp_str)
            .map_err(|err| {
                anyhow!(
                    "tradermade parsing historical resp to json: {}, err: {}",
                    &resp_str,
                    err
                )
            })
            .as_internal_err()?;

        let ret = (base, ret);

        Ok(ret.try_into()?)
    }
}

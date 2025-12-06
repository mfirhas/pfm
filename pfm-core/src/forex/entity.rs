use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use uuid::Uuid;

use super::{currency::Currency, interface::ForexError, money::Money};
use crate::error::BaseError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatesResponse<T> {
    #[serde(alias = "id")]
    pub id: Uuid,

    #[serde(alias = "source")]
    pub source: String,

    #[serde(alias = "poll_date")]
    pub poll_date: DateTime<Utc>,

    #[serde(alias = "data")]
    pub data: T,

    #[serde(alias = "error")]
    pub error: Option<String>,
}

impl<T> RatesResponse<T>
where
    T: for<'a> Deserialize<'a> + Serialize + Debug,
{
    pub(crate) fn new(source: String, data: T) -> Self {
        Self {
            id: Uuid::new_v4(),
            source,
            poll_date: Utc::now(),
            data,
            error: None,
        }
    }
}

impl RatesResponse<Rates> {
    pub(crate) fn err(date: DateTime<Utc>, err: ForexError) -> Self {
        Self {
            id: Uuid::new_v4(),
            source: String::default(),
            poll_date: Utc::now(),
            data: Rates {
                latest_update: date,
                base: Currency::default(),
                rates: RatesData::default(),
            },
            error: Some(err.detail()),
        }
    }
}

impl RatesResponse<HistoricalRates> {
    pub(crate) fn err(date: DateTime<Utc>, err: ForexError) -> Self {
        Self {
            id: Uuid::new_v4(),
            source: String::default(),
            poll_date: Utc::now(),
            data: HistoricalRates {
                date,
                base: Currency::default(),
                rates: RatesData::default(),
            },
            error: Some(err.detail()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Rates {
    #[serde(alias = "latest_update")]
    pub latest_update: DateTime<Utc>,

    #[serde(alias = "base")]
    pub base: Currency,

    #[serde(alias = "rates")]
    pub rates: RatesData,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HistoricalRates {
    #[serde(alias = "date")]
    pub date: DateTime<Utc>,

    #[serde(alias = "base")]
    pub base: Currency,

    #[serde(alias = "rates")]
    pub rates: RatesData,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RatesData {
    #[serde(alias = "USD", default)]
    pub usd: Decimal,

    #[serde(alias = "CAD", default)]
    pub cad: Decimal,

    #[serde(alias = "EUR", default)]
    pub eur: Decimal,

    #[serde(alias = "GBP", default)]
    pub gbp: Decimal,

    #[serde(alias = "CHF", default)]
    pub chf: Decimal,

    #[serde(alias = "RUB", default)]
    pub rub: Decimal,

    #[serde(alias = "CNY", default)]
    pub cny: Decimal,

    #[serde(alias = "JPY", default)]
    pub jpy: Decimal,

    #[serde(alias = "KRW", default)]
    pub krw: Decimal,

    #[serde(alias = "HKD", default)]
    pub hkd: Decimal,

    #[serde(alias = "IDR", default)]
    pub idr: Decimal,

    #[serde(alias = "MYR", default)]
    pub myr: Decimal,

    #[serde(alias = "SGD", default)]
    pub sgd: Decimal,

    #[serde(alias = "THB", default)]
    pub thb: Decimal,

    #[serde(alias = "SAR", default)]
    pub sar: Decimal,

    #[serde(alias = "AED", default)]
    pub aed: Decimal,

    #[serde(alias = "KWD", default)]
    pub kwd: Decimal,

    #[serde(alias = "INR", default)]
    pub inr: Decimal,

    #[serde(alias = "AUD", default)]
    pub aud: Decimal,

    #[serde(alias = "NZD", default)]
    pub nzd: Decimal,

    #[serde(alias = "XAU", default)]
    pub xau: Decimal,

    #[serde(alias = "XAG", default)]
    pub xag: Decimal,

    #[serde(alias = "XPT", default)]
    pub xpt: Decimal,

    #[serde(alias = "BTC", default)]
    pub btc: Decimal,

    #[serde(alias = "ETH", default)]
    pub eth: Decimal,

    #[serde(alias = "SOL", default)]
    pub sol: Decimal,

    #[serde(alias = "XRP", default)]
    pub xrp: Decimal,

    #[serde(alias = "ADA", default)]
    pub ada: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversionResponse {
    /// latest update of the currency of conversion target.
    pub date: DateTime<Utc>,

    /// convert from
    pub from: Money,

    /// conversion result.
    pub to: Money,

    /// result in form of USD 1,000.00
    pub code: String,

    /// result in form of $1,000.00
    pub symbol: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RatesList<T> {
    pub has_prev: bool,
    pub rates_list: Vec<T>,
    pub has_next: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Order {
    ASC,
    DESC,
}

use std::marker::PhantomData;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use uuid::Uuid;

use super::{currency::Currency, interface::ForexError, money::Money};

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
            error: Some(err.to_string()),
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
            error: Some(err.to_string()),
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

    #[serde(alias = "IDR", default)]
    pub idr: Decimal,

    #[serde(alias = "EUR", default)]
    pub eur: Decimal,

    #[serde(alias = "GBP", default)]
    pub gbp: Decimal,

    #[serde(alias = "JPY", default)]
    pub jpy: Decimal,

    #[serde(alias = "CHF", default)]
    pub chf: Decimal,

    #[serde(alias = "SGD", default)]
    pub sgd: Decimal,

    #[serde(alias = "CNY", default)]
    pub cny: Decimal,

    #[serde(alias = "SAR", default)]
    pub sar: Decimal,

    #[serde(alias = "XAU", default)]
    pub xau: Decimal,

    #[serde(alias = "XAG", default)]
    pub xag: Decimal,

    #[serde(alias = "XPT", default)]
    pub xpt: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversionResponse {
    /// latest update of the currency of conversion target.
    pub last_update: DateTime<Utc>,

    /// conversion result.
    pub money: Money,
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

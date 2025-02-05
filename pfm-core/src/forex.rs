// This is Interface for foreign exchange implementations.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

use anyhow::anyhow;
use chrono::{DateTime, Utc};
use iso_currency::Currency;
use lazy_static::lazy_static;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::forex_impl::utils::{parse_str, to_string};

lazy_static! {
    /// List of currencies using comma to separate thousands.
    pub static ref COMMA_SEPARATED_CURRENCIES: Vec<Currency> = {
        let comma_separated_currencies = vec![
                Currency::AUD, // Australia
                Currency::BWP, // Botswana
                Currency::XCD, // British West Indies (East Caribbean Dollar)
                Currency::BND, // Brunei
                Currency::CAD, // Canada (English-speaking)
                Currency::DOP, // Dominican Republic
                Currency::GTQ, // Guatemala
                Currency::HKD, // Hong Kong
                Currency::INR, // India
                Currency::EUR, // euro
                Currency::ILS, // Israel
                Currency::JPY, // Japan
                Currency::KES, // Kenya
                Currency::KPW, // Korea (North)
                Currency::KRW, // Korea (South)
                Currency::LBP, // Lebanon
                Currency::MYR, // Malaysia
                Currency::EUR, // Malta
                Currency::MXN, // Mexico
                Currency::NPR, // Nepal
                Currency::NZD, // New Zealand
                Currency::NIO, // Nicaragua
                Currency::NGN, // Nigeria
                Currency::PKR, // Pakistan
                Currency::CNY, // People's Republic of China
                Currency::PHP, // Philippines
                Currency::SGD, // Singapore
                Currency::LKR, // Sri Lanka
                Currency::CHF, // Switzerland (only when amount is in Swiss francs)
                Currency::TWD, // Taiwan
                Currency::TZS, // Tanzania
                Currency::THB, // Thailand
                Currency::UGX, // Uganda
                Currency::GBP, // United Kingdom
                Currency::USD, // United States (including insular areas)
                Currency::ZWL, // Zimbabwe
            ];

            comma_separated_currencies
    };
}

/// thousands separated by commas. E.g. 1,000 or 1,000.00
pub(crate) const COMMA_SEPARATOR: &str = r"^\d{1,3}(,?\d{3})*(\.\d{2})?$";

/// thousands separated by dots. E.g. 1.000 or 1.000,00
pub(crate) const DOT_SEPARATOR: &str = r"^\d{1,3}(\.?\d{3})*(?:,\d{2})?$";

lazy_static! {
    pub static ref COMMA_SEPARATOR_REGEX: regex::Regex =
        regex::Regex::new(COMMA_SEPARATOR).expect("failed compiling comma separator regex");
    pub static ref DOT_SEPARATOR_REGEX: regex::Regex =
        regex::Regex::new(DOT_SEPARATOR).expect("failed to compile dot separator regex");
}

pub(crate) const ERROR_CURRENCY_PARTS: &str = "The money must be written in ISO 4217 format using currency code first then major unit along with minor unit(optional). They're separated by space. For example: USD 5,000,000 or USD 5,000,000.23 or IDR 5.000.000 or IDR 5.000.000,00. Thousands separators are optional.";

pub(crate) const ERROR_INVALID_AMOUNT_FORMAT: &str = "The amount may contains thousands separator or not, if it contains use the appropriate ones for the currency. If minor unit exists use the correct separator.";

const ERROR_PREFIX: &str = "[FOREX]";

enum Operations {
    Add,
    Substract,
    Multiply,
    Divide,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Money {
    IDR(Decimal),
    USD(Decimal),
    EUR(Decimal),
    GBP(Decimal),
    JPY(Decimal),
    CHF(Decimal),
    SGD(Decimal),
    CNY(Decimal),
    SAR(Decimal),
}

impl Money {
    pub fn new(currency: &str, amount: &str) -> ForexResult<Self> {
        let curr = Currency::from_str(currency)
            .map_err(|err| anyhow!("{} invalid currency: {}", ERROR_PREFIX, err))?;
        let val = Decimal::from_str(amount)
            .map_err(|err| anyhow!("{} invalid amount: {}", ERROR_PREFIX, err))?;

        match curr {
            Currency::IDR => Ok(Self::IDR(val)),
            Currency::USD => Ok(Self::USD(val)),
            Currency::EUR => Ok(Self::EUR(val)),
            Currency::GBP => Ok(Self::GBP(val)),
            Currency::JPY => Ok(Self::JPY(val)),
            Currency::CHF => Ok(Self::CHF(val)),
            Currency::SGD => Ok(Self::SGD(val)),
            Currency::CNY => Ok(Self::CNY(val)),
            Currency::SAR => Ok(Self::SAR(val)),
            _ => Err(anyhow!(
                "{} Currency {} not supported",
                ERROR_PREFIX,
                curr.code()
            )),
        }
    }

    pub fn amount(&self) -> Decimal {
        match self {
            Self::IDR(val) => *val,
            Self::USD(val) => *val,
            Self::EUR(val) => *val,
            Self::GBP(val) => *val,
            Self::JPY(val) => *val,
            Self::CHF(val) => *val,
            Self::SGD(val) => *val,
            Self::CNY(val) => *val,
            Self::SAR(val) => *val,
        }
    }

    pub fn code(&self) -> String {
        match self {
            Self::IDR(_) => Currency::IDR.code().to_string(),
            Self::USD(_) => Currency::USD.code().to_string(),
            Self::EUR(_) => Currency::EUR.code().to_string(),
            Self::GBP(_) => Currency::GBP.code().to_string(),
            Self::JPY(_) => Currency::JPY.code().to_string(),
            Self::CHF(_) => Currency::CHF.code().to_string(),
            Self::SGD(_) => Currency::SGD.code().to_string(),
            Self::CNY(_) => Currency::CNY.code().to_string(),
            Self::SAR(_) => Currency::SAR.code().to_string(),
        }
    }

    pub fn symbol(&self) -> String {
        match self {
            Self::IDR(_) => Currency::IDR.symbol().to_string(),
            Self::USD(_) => Currency::USD.symbol().to_string(),
            Self::EUR(_) => Currency::EUR.symbol().to_string(),
            Self::GBP(_) => Currency::GBP.symbol().to_string(),
            Self::JPY(_) => Currency::JPY.symbol().to_string(),
            Self::CHF(_) => Currency::CHF.symbol().to_string(),
            Self::SGD(_) => Currency::SGD.symbol().to_string(),
            Self::CNY(_) => Currency::CNY.symbol().to_string(),
            Self::SAR(_) => Currency::SAR.symbol().to_string(),
        }
    }
}

impl FromStr for Money {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ret = parse_str(s)
            .map_err(|err| anyhow!("{} Failed parsing money from str: {}", ERROR_PREFIX, err))?;
        Ok(ret)
    }
}

impl Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ret = to_string(false, *self);
        write!(f, "{}", ret)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rates {
    #[serde(alias = "date")]
    pub date: DateTime<Utc>,

    #[serde(alias = "rates")]
    pub rates: Currencies,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Currencies {
    #[serde(alias = "IDR")]
    pub idr: Decimal,

    #[serde(alias = "USD")]
    pub usd: Decimal,

    #[serde(alias = "EUR")]
    pub eur: Decimal,

    #[serde(alias = "GBP")]
    pub gbp: Decimal,

    #[serde(alias = "JPY")]
    pub jpy: Decimal,

    #[serde(alias = "CHF")]
    pub chf: Decimal,

    #[serde(alias = "SGD")]
    pub sgd: Decimal,

    #[serde(alias = "CNY")]
    pub cny: Decimal,

    #[serde(alias = "SAR")]
    pub sar: Decimal,

    #[serde(alias = "XAU")]
    pub xau: Decimal,

    #[serde(alias = "XAG")]
    pub xag: Decimal,

    #[serde(alias = "XPT")]
    pub xpt: Decimal,
}

pub struct Conversion {
    /// latest update of the currency of conversion target.
    pub last_update: DateTime<Utc>,

    /// conversion result.
    pub money: Money,
}

pub type ForexResult<T> = Result<T, anyhow::Error>;

/////////////// INVOKED FROM SERVER
#[async_trait]
pub trait ForexConverter {
    /// convert from Money into to Currency using latest rates
    async fn convert(&self, from: Money, to: Currency) -> ForexResult<Conversion>;
}
///////////////

/////////////// INVOKED FROM CRON JOB
#[async_trait]
pub trait ForexRates {
    /// get latest list of rates with a base currency
    async fn rates(&self, base: Currency) -> ForexResult<Rates>;
}

#[async_trait]
pub trait ForexHistoricalRates {
    /// get historical daily rates
    async fn historical_rates(&self, date: DateTime<Utc>, base: Currency) -> ForexResult<Rates>;
}
///////////////

/////////////// INVOKED FROM SERVER
/// Interface for storing forex data fetched from 3rd APIs.
#[async_trait]
pub trait ForexStorage {
    /// insert latest rate fetched from API
    /// @date: the datetime in UTC when the data fetched.
    /// @rates: the rates to be saved.
    async fn insert_latest(&self, date: DateTime<Utc>, rates: Rates) -> ForexResult<()>;

    /// get the latest data fetched from API
    async fn get_latest(&self) -> ForexResult<Rates>;

    /// insert historical rates
    /// @date: the datetime in UTC when the data fetched.
    /// @rates: the rates to be saved.
    async fn insert_historical(&self, date: DateTime<Utc>, rates: Rates) -> ForexResult<()>;

    /// get historical rates
    async fn get_historical(&self) -> ForexResult<Rates>;
}
///////////////

//////////
// APIs //
//////////

/// Convert Money into another currency.
/// This only call storage to get latest rates and do the calculations.
pub async fn convert<FS>(forex: &FS, from: Money, to: &str) -> ForexResult<Conversion>
where
    FS: ForexStorage,
{
    if from.amount() == dec!(0) {
        return Err(anyhow!("[FOREX] amount conversion must be greater than 0"));
    }

    Money::new(to, "1").map_err(|err| {
        anyhow!(
            "{} Failed doing conversion: target currency is invalid: {}",
            ERROR_PREFIX,
            err
        )
    })?;

    let ret = todo!("do conversion by reading latest data, and do the calculation");

    Ok(ret)
}

/// Get rates from 3rd API.
/// Invoked from Cron service.
pub async fn get_rates<FX>(forex: &FX, base: Currency, to: &[Currency]) -> ForexResult<Rates>
where
    FX: ForexRates,
{
    let ret = forex.rates(base).await?;

    Ok(ret)
}

/// Get historical rates from 3rd API.
/// Invoked from Cron service.
pub async fn get_historical_rates<FX>(
    forex: &FX,
    date: DateTime<Utc>,
    base: Currency,
    to: &[Currency],
) -> ForexResult<Rates>
where
    FX: ForexHistoricalRates,
{
    let ret = forex.historical_rates(date, base).await?;

    Ok(ret)
}

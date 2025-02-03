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

enum Operations {
    Add,
    Substract,
    Multiply,
    Divide,
}

#[derive(Debug)]
pub struct Money {
    pub currency: Currency,
    pub amount: Decimal,
}

impl Money {
    pub fn new(currency: &str, amount: &str) -> ForexResult<Self> {
        let curr = Currency::from_str(currency)
            .map_err(|err| anyhow!(format!("invalid currency: {}", err)))?;
        let val =
            Decimal::from_str(amount).map_err(|err| anyhow!(format!("invalid amount: {}", err)))?;

        Ok(Self {
            currency: curr,
            amount: val,
        })
    }

    pub async fn add<FX>(&self, forex: &FX, rhs: Money) -> ForexResult<Money>
    where
        FX: ForexConverter,
    {
        Self::operate(forex, Operations::Add, self, &rhs).await
    }

    pub async fn substract<FX>(&self, forex: &FX, rhs: Money) -> ForexResult<Money>
    where
        FX: ForexConverter,
    {
        Self::operate(forex, Operations::Substract, self, &rhs).await
    }

    pub async fn multiply<FX>(&self, forex: &FX, rhs: Money) -> ForexResult<Money>
    where
        FX: ForexConverter,
    {
        Self::operate(forex, Operations::Multiply, self, &rhs).await
    }

    pub async fn divide<FX>(&self, forex: &FX, rhs: Money) -> ForexResult<Money>
    where
        FX: ForexConverter,
    {
        Self::operate(forex, Operations::Divide, self, &rhs).await
    }

    async fn operate<FX>(
        forex: &FX,
        operation: Operations,
        lhs: &Money,
        rhs: &Money,
    ) -> ForexResult<Money>
    where
        FX: ForexConverter,
    {
        let mut _rhs: Money;
        if lhs.currency != rhs.currency {
            let ret = forex.convert(rhs, lhs.currency).await?;
            let money = ret.money;
            _rhs = money;
        } else {
            _rhs = Money {
                currency: rhs.currency,
                amount: rhs.amount,
            }
        }

        let (currency, amount) = match operation {
            Operations::Add => (lhs.currency, lhs.amount + _rhs.amount),
            Operations::Substract => (lhs.currency, lhs.amount - _rhs.amount),
            Operations::Multiply => (lhs.currency, lhs.amount * _rhs.amount),
            Operations::Divide => (lhs.currency, lhs.amount / _rhs.amount),
        };

        Ok(Money { currency, amount })
    }
}

impl FromStr for Money {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(parse_str(s)?)
    }
}

impl Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let money_display = to_string(false, self);
        write!(f, "{}", money_display)
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

/////////////// INVOKED FROM CRON JOB
#[async_trait]
pub trait ForexConverter {
    /// convert from Money into to Currency using latest rates
    async fn convert(&self, from: &Money, to: Currency) -> ForexResult<Conversion>;
}

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
#[async_trait]
pub trait ForexStorage {
    /// insert latest rate fetched from API
    /// @date: the datetime in UTC when the data fetched.
    /// @rates: the rates to be saved.
    async fn insert_latest(&self, date: DateTime<Utc>, rates: Rates) -> ForexResult<()>;

    /// get the latest data fetched from API
    async fn get_latest(&self) -> ForexResult<Rates>;
}
///////////////

//////////
// APIs //
//////////

/// Convert Money into another currency.
/// This only call storage to get latest rates and do the calculations.
pub async fn convert<FS>(forex: &FS, from: Money, to: Currency) -> ForexResult<Conversion>
where
    FS: ForexStorage,
{
    if from.amount == dec!(0) {
        return Err(anyhow!("[FOREX] amount conversion must be greater than 0"));
    }

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

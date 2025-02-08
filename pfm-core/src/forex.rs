// This is Interface for foreign exchange implementations.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use anyhow::anyhow;
use chrono::{DateTime, Utc};
use iso_currency::Currency;
use lazy_static::lazy_static;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::{
    forex_impl::utils::{parse_str, to_string},
    global,
};

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

/// List of supported currencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Currencies {
    IDR,
    USD,
    EUR,
    GBP,
    JPY,
    CHF,
    SGD,
    CNY,
    SAR,
}

impl Currencies {
    pub fn code(&self) -> &'static str {
        match self {
            Self::IDR => Currency::IDR.code(),
            Self::USD => Currency::USD.code(),
            Self::EUR => Currency::EUR.code(),
            Self::GBP => Currency::GBP.code(),
            Self::JPY => Currency::JPY.code(),
            Self::CHF => Currency::CHF.code(),
            Self::SGD => Currency::SGD.code(),
            Self::CNY => Currency::CNY.code(),
            Self::SAR => Currency::SAR.code(),
        }
    }
}

impl From<Money> for Currencies {
    fn from(value: Money) -> Self {
        match value {
            Money::IDR(_) => Self::IDR,
            Money::USD(_) => Self::USD,
            Money::EUR(_) => Self::EUR,
            Money::GBP(_) => Self::GBP,
            Money::JPY(_) => Self::JPY,
            Money::CHF(_) => Self::CHF,
            Money::SGD(_) => Self::SGD,
            Money::CNY(_) => Self::CNY,
            Money::SAR(_) => Self::SAR,
        }
    }
}

impl PartialEq<Currencies> for Money {
    fn eq(&self, other: &Currencies) -> bool {
        match (self, other) {
            (Money::IDR(_), Currencies::IDR) => true,
            (Money::USD(_), Currencies::USD) => true,
            (Money::EUR(_), Currencies::EUR) => true,
            (Money::GBP(_), Currencies::GBP) => true,
            (Money::JPY(_), Currencies::JPY) => true,
            (Money::CHF(_), Currencies::CHF) => true,
            (Money::SGD(_), Currencies::SGD) => true,
            (Money::CNY(_), Currencies::CNY) => true,
            (Money::SAR(_), Currencies::SAR) => true,
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Deserialize)]
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
        let ret = to_string(global::config().forex_use_symbol, *self);
        write!(f, "{}", ret)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RatesResponse<T> {
    #[serde(alias = "source")]
    source: String,

    #[serde(alias = "data")]
    data: T,

    #[serde(alias = "error")]
    error: Option<String>,
}

impl<T> RatesResponse<T>
where
    T: for<'a> Deserialize<'a> + Serialize + Debug,
{
    pub(crate) fn new(source: String, data: T) -> Self {
        Self {
            source,
            data,
            error: None,
        }
    }
}

impl RatesResponse<Rates> {
    // TODO: add err handling to api calls
    pub(crate) fn err(source: String, error: String) -> Self {
        Self {
            source,
            data: Rates::default(),
            error: Some(error),
        }
    }
}

impl RatesResponse<HistoricalRates> {
    // TODO: add err handling to api calls
    pub(crate) fn err(source: String, error: String) -> Self {
        Self {
            source,
            data: HistoricalRates::default(),
            error: Some(error),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Rates {
    #[serde(alias = "latest_update")]
    pub latest_update: DateTime<Utc>,

    #[serde(alias = "rates")]
    pub rates: RatesData,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct HistoricalRates {
    #[serde(alias = "date")]
    pub date: DateTime<Utc>,

    #[serde(alias = "rates")]
    pub rates: RatesData,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RatesData {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversionResponse {
    /// latest update of the currency of conversion target.
    pub last_update: DateTime<Utc>,

    /// conversion result.
    pub money: Money,
}

pub type ForexResult<T> = Result<T, anyhow::Error>;

///////////////////////////////////// INTERFACES /////////////////////////////////////
/////////////// INVOKED FROM SERVER
/// ForexConverter is interface for 3rd API converting amount from 1 currency into another.
/// NOTE: for now use storage using rates fetched and calculate from there.
#[async_trait]
pub trait ForexConverter {
    /// convert from Money into to Currency using latest rates
    async fn convert(&self, from: Money, to: Currencies) -> ForexResult<ConversionResponse>;
}
///////////////

/////////////// INVOKED FROM CRON JOB
#[async_trait]
pub trait ForexRates {
    /// get latest list of rates with a base currency
    async fn rates(&self, base: Currencies) -> ForexResult<RatesResponse<Rates>>;
}

#[async_trait]
pub trait ForexHistoricalRates {
    /// get historical daily rates
    async fn historical_rates(
        &self,
        date: DateTime<Utc>,
        base: Currencies,
    ) -> ForexResult<RatesResponse<HistoricalRates>>;
}
///////////////

/////////////// INVOKED FROM SERVER
/// Interface for storing forex data fetched from 3rd APIs.
#[async_trait]
pub trait ForexStorage {
    /// insert latest rate fetched from API
    /// @date: the datetime in UTC when the data fetched.
    /// @rates: the rates to be saved.
    async fn insert_latest(
        &self,
        date: DateTime<Utc>,
        rates: &RatesResponse<Rates>,
    ) -> ForexResult<()>;

    /// get the latest data fetched from API
    async fn get_latest(&self) -> ForexResult<RatesResponse<Rates>>;

    /// insert historical rates
    /// @date: the datetime in UTC when the data fetched.
    /// @rates: the rates to be saved.
    async fn insert_historical(
        &self,
        date: DateTime<Utc>,
        rates: &RatesResponse<HistoricalRates>,
    ) -> ForexResult<()>;

    /// get historical rates
    async fn get_historical(
        &self,
        date: DateTime<Utc>,
    ) -> ForexResult<RatesResponse<HistoricalRates>>;
}
///////////////
///////////////////////////////////// INTERFACES(END) /////////////////////////////////////

///////////////////////////////////// APIs /////////////////////////////////////
/// Convert Money into another currency.
/// This only call storage to get latest rates and do the calculations.
pub async fn convert<FS>(
    storage: &FS,
    from: Money,
    to: Currencies,
) -> ForexResult<ConversionResponse>
where
    FS: ForexStorage,
{
    if from.amount() == dec!(0) {
        return Err(anyhow!("[FOREX] amount conversion must be greater than 0"));
    }

    if from == to {
        return Ok(ConversionResponse {
            last_update: Utc::now(),
            money: from,
        });
    }

    let latest_rates = storage.get_latest().await.map_err(|err| {
        anyhow!(
            "{} failed getting latest rates from storage: {}",
            ERROR_PREFIX,
            err
        )
    })?;

    // todo: conversion

    let ret = todo!("do conversion by reading latest data from storage, and do the calculation");

    Ok(ret)
}

/// Get rates from 3rd API.
/// Invoked from Cron service.
pub async fn get_rates<FX, FS>(
    forex: &FX,
    storage: &FS,
    base: Currencies,
) -> ForexResult<RatesResponse<Rates>>
where
    FX: ForexRates,
    FS: ForexStorage,
{
    let ret = forex.rates(base).await?;

    storage.insert_latest(ret.data.latest_update, &ret).await?;

    Ok(ret)
}

/// Get historical rates from 3rd API.
/// Invoked from Cron service.
pub async fn get_historical_rates<FX, FS>(
    forex: &FX,
    storage: &FS,
    date: DateTime<Utc>,
    base: Currencies,
) -> ForexResult<RatesResponse<HistoricalRates>>
where
    FX: ForexHistoricalRates,
    FS: ForexStorage,
{
    let ret = forex.historical_rates(date, base).await?;

    storage.insert_historical(ret.data.date, &ret).await?;

    Ok(ret)
}
///////////////////////////////////// APIs(END) /////////////////////////////////////

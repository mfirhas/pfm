// forex.rs is domain for the exchanges/prices of currencies, precious metals, cryptos, etc.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
    str::FromStr,
};
use uuid::Uuid;

use anyhow::anyhow;
use chrono::{DateTime, Utc};
use iso_currency::Currency as CurrencyLib;
use lazy_static::lazy_static;
use rust_decimal::Decimal;

use crate::{
    forex_impl::utils::{convert_currency, parse_str, to_string},
    global,
};

lazy_static! {
    /// Using ISO 4217 currency code with comma separated thousands(optional) and dot separated fraction(optional).
    /// e.g.
    /// USD 1000
    /// USD 1,000
    /// USD 1,000.00
    /// IDR 5,000.235
    /// IDR 5,000,0223.445
    pub(crate) static ref MONEY_FORMAT_REGEX: regex::Regex =
        regex::Regex::new(r"^([A-Z]{3})\s+((?:\d{1,3}(?:,\d{3})*|\d+)(?:\.\d+)?)$").expect("failed compiling money format regex");
}

pub(crate) const ERROR_MONEY_FORMAT: &str = "The money must be written in ISO 4217 format: <CODE> <AMOUNT>. Amount may be separated by comma for thousands, and by dot for fraction.";

const ERROR_PREFIX: &str = "[FOREX]";

/// pfm-core Currency.
/// List of supported currencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Currency {
    USD,
    IDR,
    EUR,
    GBP,
    JPY,
    CHF,
    SGD,
    CNY,
    SAR,
}

impl FromStr for Currency {
    type Err = ForexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let curr: CurrencyLib = s.parse().map_err(|err| {
            ForexError::InputError(anyhow!("{} invalid currency: {}", ERROR_PREFIX, err))
        })?;

        match curr {
            CurrencyLib::IDR => Ok(Self::IDR),
            CurrencyLib::USD => Ok(Self::USD),
            CurrencyLib::EUR => Ok(Self::EUR),
            CurrencyLib::GBP => Ok(Self::GBP),
            CurrencyLib::JPY => Ok(Self::JPY),
            CurrencyLib::CHF => Ok(Self::CHF),
            CurrencyLib::SGD => Ok(Self::SGD),
            CurrencyLib::CNY => Ok(Self::CNY),
            CurrencyLib::SAR => Ok(Self::SAR),
            _ => Err(ForexError::InputError(anyhow!(
                "{} Currency {} not supported",
                ERROR_PREFIX,
                curr.code()
            ))),
        }
    }
}

impl Default for Currency {
    fn default() -> Self {
        Self::USD
    }
}

impl Currency {
    pub fn code(&self) -> &'static str {
        match self {
            Self::IDR => CurrencyLib::IDR.code(),
            Self::USD => CurrencyLib::USD.code(),
            Self::EUR => CurrencyLib::EUR.code(),
            Self::GBP => CurrencyLib::GBP.code(),
            Self::JPY => CurrencyLib::JPY.code(),
            Self::CHF => CurrencyLib::CHF.code(),
            Self::SGD => CurrencyLib::SGD.code(),
            Self::CNY => CurrencyLib::CNY.code(),
            Self::SAR => CurrencyLib::SAR.code(),
        }
    }
}

impl From<Money> for Currency {
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

impl Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let r = match self {
            Self::IDR => CurrencyLib::IDR.code(),
            Self::USD => CurrencyLib::USD.code(),
            Self::EUR => CurrencyLib::EUR.code(),
            Self::GBP => CurrencyLib::GBP.code(),
            Self::JPY => CurrencyLib::JPY.code(),
            Self::CHF => CurrencyLib::CHF.code(),
            Self::SGD => CurrencyLib::SGD.code(),
            Self::CNY => CurrencyLib::CNY.code(),
            Self::SAR => CurrencyLib::SAR.code(),
        };

        write!(f, "{}", r)
    }
}

impl PartialEq<Currency> for Money {
    fn eq(&self, other: &Currency) -> bool {
        match (self, other) {
            (Money::IDR(_), Currency::IDR) => true,
            (Money::USD(_), Currency::USD) => true,
            (Money::EUR(_), Currency::EUR) => true,
            (Money::GBP(_), Currency::GBP) => true,
            (Money::JPY(_), Currency::JPY) => true,
            (Money::CHF(_), Currency::CHF) => true,
            (Money::SGD(_), Currency::SGD) => true,
            (Money::CNY(_), Currency::CNY) => true,
            (Money::SAR(_), Currency::SAR) => true,
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Deserialize)]
pub enum Money {
    USD(Decimal),
    IDR(Decimal),
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
        let curr = CurrencyLib::from_str(currency).map_err(|err| {
            ForexError::InputError(anyhow!("{} invalid currency: {}", ERROR_PREFIX, err))
        })?;
        let val = Decimal::from_str(amount).map_err(|err| {
            ForexError::InputError(anyhow!("{} invalid amount: {}", ERROR_PREFIX, err))
        })?;

        match curr {
            CurrencyLib::IDR => Ok(Self::IDR(val)),
            CurrencyLib::USD => Ok(Self::USD(val)),
            CurrencyLib::EUR => Ok(Self::EUR(val)),
            CurrencyLib::GBP => Ok(Self::GBP(val)),
            CurrencyLib::JPY => Ok(Self::JPY(val)),
            CurrencyLib::CHF => Ok(Self::CHF(val)),
            CurrencyLib::SGD => Ok(Self::SGD(val)),
            CurrencyLib::CNY => Ok(Self::CNY(val)),
            CurrencyLib::SAR => Ok(Self::SAR(val)),
            _ => Err(ForexError::InputError(anyhow!(
                "{} Currency {} not supported",
                ERROR_PREFIX,
                curr.code()
            ))),
        }
    }

    pub fn new_money(currency: Currency, amount: Decimal) -> Money {
        match currency {
            Currency::IDR => Money::IDR(amount),
            Currency::USD => Money::USD(amount),
            Currency::EUR => Money::EUR(amount),
            Currency::GBP => Money::GBP(amount),
            Currency::JPY => Money::JPY(amount),
            Currency::CHF => Money::CHF(amount),
            Currency::SGD => Money::SGD(amount),
            Currency::CNY => Money::CNY(amount),
            Currency::SAR => Money::SAR(amount),
        }
    }

    pub fn currency(&self) -> Currency {
        match self {
            Self::IDR(_) => Currency::IDR,
            Self::USD(_) => Currency::USD,
            Self::EUR(_) => Currency::EUR,
            Self::GBP(_) => Currency::GBP,
            Self::JPY(_) => Currency::JPY,
            Self::CHF(_) => Currency::CHF,
            Self::SGD(_) => Currency::SGD,
            Self::CNY(_) => Currency::CNY,
            Self::SAR(_) => Currency::SAR,
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
            Self::IDR(_) => CurrencyLib::IDR.code().to_string(),
            Self::USD(_) => CurrencyLib::USD.code().to_string(),
            Self::EUR(_) => CurrencyLib::EUR.code().to_string(),
            Self::GBP(_) => CurrencyLib::GBP.code().to_string(),
            Self::JPY(_) => CurrencyLib::JPY.code().to_string(),
            Self::CHF(_) => CurrencyLib::CHF.code().to_string(),
            Self::SGD(_) => CurrencyLib::SGD.code().to_string(),
            Self::CNY(_) => CurrencyLib::CNY.code().to_string(),
            Self::SAR(_) => CurrencyLib::SAR.code().to_string(),
        }
    }

    pub fn symbol(&self) -> String {
        match self {
            Self::IDR(_) => CurrencyLib::IDR.symbol().to_string(),
            Self::USD(_) => CurrencyLib::USD.symbol().to_string(),
            Self::EUR(_) => CurrencyLib::EUR.symbol().to_string(),
            Self::GBP(_) => CurrencyLib::GBP.symbol().to_string(),
            Self::JPY(_) => CurrencyLib::JPY.symbol().to_string(),
            Self::CHF(_) => CurrencyLib::CHF.symbol().to_string(),
            Self::SGD(_) => CurrencyLib::SGD.symbol().to_string(),
            Self::CNY(_) => CurrencyLib::CNY.symbol().to_string(),
            Self::SAR(_) => CurrencyLib::SAR.symbol().to_string(),
        }
    }
}

impl FromStr for Money {
    type Err = ForexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ret = parse_str(s)?;
        Ok(ret)
    }
}

impl Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ret = to_string(global::config().forex_use_symbol, *self);
        write!(f, "{}", ret)
    }
}

/// used as common response for http service
#[derive(Debug, Serialize, Deserialize)]
pub struct HttpResponse<T> {
    #[serde(rename = "data")]
    pub data: Option<T>,

    #[serde(rename = "error")]
    pub error: Option<String>,

    #[serde(skip)]
    _marker: PhantomData<T>,
}

impl<T> HttpResponse<T> {
    pub fn new(data: T) -> Self {
        Self {
            data: Some(data),
            error: None,
            _marker: PhantomData,
        }
    }

    pub fn err(error: String) -> Self {
        Self {
            data: None,
            error: Some(error),
            _marker: PhantomData,
        }
    }
}

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
        let (source, error) = match err {
            ForexError::InputError(err) => ("forex".to_string(), err.to_string()),
            ForexError::StorageError(err) => ("storage".to_string(), err.to_string()),
            ForexError::ExchangeAPIError(err) => (
                "https://github.com/fawazahmed0/exchange-api/".to_string(),
                err.to_string(),
            ),
            ForexError::CurrencyAPIError(err) => ("currencyapi.com".to_string(), err.to_string()),
            ForexError::OpenExchangeAPIError(err) => {
                ("openexchangerates.org".to_string(), err.to_string())
            }
        };
        Self {
            id: Uuid::new_v4(),
            source,
            poll_date: Utc::now(),
            data: Rates {
                latest_update: date,
                base: Currency::default(),
                rates: RatesData::default(),
            },
            error: Some(error),
        }
    }
}

impl RatesResponse<HistoricalRates> {
    pub(crate) fn err(date: DateTime<Utc>, err: ForexError) -> Self {
        let (source, error) = match err {
            ForexError::InputError(err) => ("forex".to_string(), err.to_string()),
            ForexError::StorageError(err) => ("storage".to_string(), err.to_string()),
            ForexError::ExchangeAPIError(err) => (
                "https://github.com/fawazahmed0/exchange-api/".to_string(),
                err.to_string(),
            ),
            ForexError::CurrencyAPIError(err) => ("currencyapi.com".to_string(), err.to_string()),
            ForexError::OpenExchangeAPIError(err) => {
                ("openexchangerates.org".to_string(), err.to_string())
            }
        };
        Self {
            id: Uuid::new_v4(),
            source,
            poll_date: Utc::now(),
            data: HistoricalRates {
                date,
                base: Currency::default(),
                rates: RatesData::default(),
            },
            error: Some(error),
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
    #[serde(alias = "USD")]
    pub usd: Decimal,

    #[serde(alias = "IDR")]
    pub idr: Decimal,

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RatesList<T> {
    pub has_prev: bool,
    pub rates_list: Vec<T>,
    pub has_next: bool,
}

pub type ForexResult<T> = Result<T, ForexError>;

#[derive(Debug)]
pub enum ForexError {
    InputError(anyhow::Error),
    StorageError(anyhow::Error),
    ExchangeAPIError(anyhow::Error),
    CurrencyAPIError(anyhow::Error),
    OpenExchangeAPIError(anyhow::Error),
}

impl Display for ForexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ret = match self {
            Self::InputError(val) => val.to_string(),
            Self::StorageError(val) => val.to_string(),
            Self::ExchangeAPIError(val) => val.to_string(),
            Self::CurrencyAPIError(val) => val.to_string(),
            Self::OpenExchangeAPIError(val) => val.to_string(),
        };
        write!(f, "{}", ret)
    }
}

///////////////////////////////////// INTERFACES /////////////////////////////////////
/////////////// INVOKED FROM SERVER and APP
/// ForexConverter is interface for 3rd API converting amount from 1 currency into another.
/// NOTE: for now use storage using rates fetched and calculate from there.
#[async_trait]
pub trait ForexConverter {
    /// convert from Money into to Currency using latest rates
    async fn convert(&self, from: Money, to: Currency) -> ForexResult<ConversionResponse>;
}
///////////////

/////////////// INVOKED FROM CRON JOB
#[async_trait]
pub trait ForexRates {
    /// get latest list of rates with a base currency
    async fn rates(&self, base: Currency) -> ForexResult<RatesResponse<Rates>>;
}

#[async_trait]
pub trait ForexHistoricalRates {
    /// get historical daily rates
    async fn historical_rates(
        &self,
        date: DateTime<Utc>,
        base: Currency,
    ) -> ForexResult<RatesResponse<HistoricalRates>>;
}
///////////////

/////////////// INVOKED FROM HTTP and CRON SERVICE, and APP.
/// Interface for storing forex data fetched from 3rd APIs.
#[async_trait]
pub trait ForexStorage {
    /// insert latest rate fetched from API
    /// @date: the datetime in UTC when the data fetched.
    /// @rates: the rates to be saved.
    async fn insert_latest<T>(
        &self,
        date: DateTime<Utc>,
        rates: &RatesResponse<T>,
    ) -> ForexResult<()>
    where
        T: Debug + Serialize + for<'de> Deserialize<'de> + Send + Sync;

    /// get the latest data fetched from API
    async fn get_latest(&self) -> ForexResult<RatesResponse<Rates>>;

    /// insert historical rates
    /// @date: the datetime in UTC when the data fetched.
    /// @rates: the rates to be saved.
    async fn insert_historical<T>(
        &self,
        date: DateTime<Utc>,
        rates: &RatesResponse<T>,
    ) -> ForexResult<()>
    where
        T: Debug + Serialize + for<'de> Deserialize<'de> + Send + Sync;

    /// get historical rates
    async fn get_historical(
        &self,
        date: DateTime<Utc>,
    ) -> ForexResult<RatesResponse<HistoricalRates>>;

    /// get list of latest rates returning list and has next or not
    async fn get_latest_list(
        &self,
        page: u32,
        size: u32,
        order: Order,
    ) -> ForexResult<RatesList<RatesResponse<Rates>>>;

    /// get list of historical rates returning list and has next or not
    async fn get_historical_list(
        &self,
        page: u32,
        size: u32,
        order: Order,
    ) -> ForexResult<RatesList<RatesResponse<HistoricalRates>>>;
}
///////////////

////////////// STORAGE INFO
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Order {
    ASC,
    DESC,
}
////////////// STORAGE INFO (END)
///////////////////////////////////// INTERFACES(END) /////////////////////////////////////

///////////////////////////////////// APIs /////////////////////////////////////
/// Convert Money into another currency.
/// This only call storage to get latest rates and do the calculations.
pub async fn convert<FS>(storage: &FS, from: Money, to: Currency) -> ForexResult<ConversionResponse>
where
    FS: ForexStorage,
{
    let latest_rates = storage.get_latest().await?;
    if let Some(err) = latest_rates.error {
        return Err(ForexError::StorageError(anyhow!(err)));
    }

    let ret = {
        let res = convert_currency(&latest_rates.data, from, to)?;
        let date = latest_rates.data.latest_update;

        ConversionResponse {
            last_update: date,
            money: res,
        }
    };

    Ok(ret)
}

pub async fn batch_convert<FS>(
    storage: &FS,
    from: Vec<Money>,
    to: Currency,
) -> ForexResult<Vec<ConversionResponse>>
where
    FS: ForexStorage,
{
    let mut results: Vec<ConversionResponse> = vec![];

    for x in from {
        let ret = convert(storage, x, to).await?;

        results.push(ret);
    }

    Ok(results)
}

/// Get rates from 3rd API.
/// Invoked from Cron service.
pub async fn poll_rates<FX, FS>(
    forex: &FX,
    storage: &FS,
    base: Currency,
) -> ForexResult<RatesResponse<Rates>>
where
    FX: ForexRates,
    FS: ForexStorage,
{
    let ret = match forex.rates(base).await {
        Ok(val) => val,
        Err(error) => RatesResponse::<Rates>::err(Utc::now(), error),
    };

    storage.insert_latest(ret.data.latest_update, &ret).await?;

    Ok(ret)
}

/// Get historical rates from 3rd API.
/// Invoked from Cron service.
pub async fn poll_historical_rates<FX, FS>(
    forex: &FX,
    storage: &FS,
    date: DateTime<Utc>,
    base: Currency,
) -> ForexResult<RatesResponse<HistoricalRates>>
where
    FX: ForexHistoricalRates,
    FS: ForexStorage,
{
    let ret = match forex.historical_rates(date, base).await {
        Ok(val) => {
            storage.insert_historical(val.data.date, &val).await?;
            val
        }
        Err(error) => {
            let err = RatesResponse::<HistoricalRates>::err(date, error);
            storage.insert_historical(date, &err).await?;
            err
        }
    };

    Ok(ret)
}
///////////////////////////////////// APIs(END) /////////////////////////////////////

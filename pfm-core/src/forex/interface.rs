use std::fmt::Debug;
use std::fmt::Display;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::currency::Currency;
use super::entity::ConversionResponse;
use super::entity::HistoricalRates;
use super::entity::Order;
use super::entity::Rates;
use super::entity::RatesList;
use super::entity::RatesResponse;
use super::money::Money;

pub(super) const ERROR_PREFIX: &str = "[FOREX]";

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

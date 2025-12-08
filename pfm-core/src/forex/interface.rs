use std::fmt::Debug;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::currency::Currency;
use super::entity::ConversionResponse;
use super::entity::Order;
use super::entity::Rates;
use super::entity::RatesList;
use super::entity::RatesResponse;
use super::money::Money;
use crate::error::Error;
use crate::error::{BaseError, ClientError, InternalError};
use thiserror::Error;

pub(super) const ERROR_PREFIX: &str = "[FOREX]";

pub type ForexResult<T> = Result<T, ForexError>;

#[derive(Debug, Error)]
pub enum ForexError {
    #[error("{ERROR_PREFIX} generic error: {0}")]
    Error(#[from] Error),

    #[error("{ERROR_PREFIX} client error: {0}")]
    ClientError(#[from] ClientError),

    #[error("{ERROR_PREFIX} internal error: {0}")]
    InternalError(#[from] InternalError),
}

impl ForexError {
    pub fn error(err_msg: &str) -> Self {
        Self::new(Error::from_msg(err_msg))
    }
    pub fn client_error(err_msg: &str) -> Self {
        ForexError::ClientError(ClientError::from_msg(err_msg))
    }

    pub fn internal_error(err_msg: &str) -> Self {
        ForexError::InternalError(InternalError::from_msg(err_msg))
    }
}

impl BaseError for ForexError {
    fn new(err: impl Into<anyhow::Error>) -> Self {
        Self::Error(Error::new(err))
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
    ) -> ForexResult<RatesResponse<Rates>>;
}

#[async_trait]
pub trait ForexTimeseriesRates {
    /// get historical rates in range of dates
    async fn timeseries_rates(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        base: Currency,
    ) -> ForexResult<Vec<RatesResponse<Rates>>>;
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
    /// @date: the datetime in UTC the date of rate.
    /// @rates: the rates to be saved.
    async fn insert_historical<T>(
        &self,
        date: DateTime<Utc>,
        rates: &RatesResponse<T>,
    ) -> ForexResult<()>
    where
        T: Debug + Serialize + for<'de> Deserialize<'de> + Send + Sync;

    //// insert historical in batch
    async fn insert_historical_batch(&self, rates: Vec<RatesResponse<Rates>>) -> ForexResult<()>;

    /// update some existing rates data with new ones
    /// new_data contains money, the currency and the values.
    async fn update_historical_rates_data(
        &self,
        date: DateTime<Utc>,
        new_data: Vec<Money>,
    ) -> ForexResult<RatesResponse<Rates>>;

    /// get historical rates
    async fn get_historical(&self, date: DateTime<Utc>) -> ForexResult<RatesResponse<Rates>>;

    /// get historical rates in range of dates
    async fn get_historical_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> ForexResult<Vec<RatesResponse<Rates>>>;

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
    ) -> ForexResult<RatesList<RatesResponse<Rates>>>;
}

#[async_trait]
pub trait ForexStorageDeletion {
    /// clear all inside forex latest directory except latest one
    async fn clear_latest(&self) -> ForexResult<()>;
}

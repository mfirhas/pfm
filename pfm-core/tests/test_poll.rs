use chrono::{TimeZone, Utc};
use pfm_core::{
    forex::{
        self,
        interface::{ForexHistoricalRates, ForexStorage, ForexTimeseriesRates},
        service::{poll_historical_rates, poll_rates},
        Money,
    },
    forex_impl::{
        self,
        forex_storage::{self, ForexStorageImpl},
        tradermade,
    },
    global::{self, BASE_CURRENCY},
};
use rust_decimal_macros::dec;

#[tokio::test]
async fn test_exchange_api_latest() {
    let http_client = global::http_client();
    let fs = global::storage_fs();
    let exchange_api_impl = forex_impl::exchange_api::Api::new(http_client);
    let storage_impl = forex_storage::ForexStorageImpl::new(fs);

    let ret = poll_rates(&exchange_api_impl, &storage_impl, BASE_CURRENCY).await;

    dbg!(&ret);

    assert!(ret.is_ok());

    assert!(ret.as_ref().unwrap().error.is_none());
}

#[tokio::test]
async fn test_exchange_api_historical() {
    let http_client = global::http_client();
    let fs = global::storage_fs();
    let exchange_api_impl = forex_impl::exchange_api::Api::new(http_client);
    let storage_impl = forex_storage::ForexStorageImpl::new(fs);

    let date = Utc.with_ymd_and_hms(2024, 6, 6, 0, 0, 0).unwrap();

    let ret = poll_historical_rates(&exchange_api_impl, &storage_impl, date, BASE_CURRENCY).await;

    dbg!(&ret);

    assert!(ret.is_ok());

    assert!(ret.as_ref().unwrap().error.is_none());
}

#[tokio::test]
async fn test_currency_api_historical() {
    let http_client = global::http_client();
    let fs = global::storage_fs();
    let exchange_api_impl =
        forex_impl::currency_api::Api::new(&global::config().forex_currency_api_key, http_client);
    let storage_impl = forex_storage::ForexStorageImpl::new(fs);

    let date = Utc.with_ymd_and_hms(2019, 6, 6, 0, 0, 0).unwrap();

    let ret = poll_historical_rates(&exchange_api_impl, &storage_impl, date, BASE_CURRENCY).await;

    dbg!(&ret);

    assert!(ret.is_ok());

    assert!(ret.as_ref().unwrap().error.is_none());
}

#[tokio::test]
async fn test_open_exchange_rates_latest() {
    let http_client = global::http_client();
    let fs = global::storage_fs();
    let exchange_api_impl = forex_impl::open_exchange_api::Api::new(
        &global::config().forex_open_exchange_api_key,
        http_client,
    );
    let storage_impl = forex_storage::ForexStorageImpl::new(fs);

    let ret = poll_rates(&exchange_api_impl, &storage_impl, BASE_CURRENCY).await;

    dbg!(&ret);

    assert!(ret.is_ok());

    assert!(ret.as_ref().unwrap().error.is_none());
}

#[tokio::test]
async fn test_open_exchange_rates_historical() {
    let http_client = global::http_client();
    let fs = global::storage_fs();
    let exchange_api_impl = forex_impl::open_exchange_api::Api::new(
        &global::config().forex_open_exchange_api_key,
        http_client,
    );
    let storage_impl = forex_storage::ForexStorageImpl::new(fs);

    let date = Utc.with_ymd_and_hms(2000, 6, 6, 0, 0, 0).unwrap();

    let ret = poll_historical_rates(&exchange_api_impl, &storage_impl, date, BASE_CURRENCY).await;

    dbg!(&ret);

    assert!(ret.is_ok());

    assert!(ret.as_ref().unwrap().error.is_none());
}

#[tokio::test]
pub async fn test_currencybeacon_latest_rates() {
    let api = forex_impl::currencybeacon::Api::new(
        &global::config().forex_currencybeacon_api_key,
        global::http_client(),
    );
    let storage = forex_storage::ForexStorageImpl::new(global::storage_fs());
    let ret = poll_rates(&api, &storage, global::BASE_CURRENCY).await;
    dbg!(&ret);

    assert!(&ret.is_ok());

    assert!(ret.as_ref().unwrap().error.is_none());
}

#[tokio::test]
pub async fn test_currencybeacon_historical_rates() {
    let api = forex_impl::currencybeacon::Api::new(
        &global::config().forex_currencybeacon_api_key,
        global::http_client(),
    );
    let storage = forex_storage::ForexStorageImpl::new(global::storage_fs());
    let date = Utc.with_ymd_and_hms(2022, 6, 6, 0, 0, 0).unwrap();
    let ret = poll_historical_rates(&api, &storage, date, global::BASE_CURRENCY).await;
    dbg!(&ret);

    assert!(&ret.is_ok());

    assert!(ret.as_ref().unwrap().error.is_none());
}

/// test currencybeacon timeseries api
#[tokio::test]
pub async fn test_currencybeacon_timeseries_rates() {
    let api = forex_impl::currencybeacon::Api::new(
        &global::config().forex_currencybeacon_api_key,
        global::http_client(),
    );

    let start_date = Utc.with_ymd_and_hms(2017, 1, 2, 0, 0, 0).unwrap();
    let end_date = Utc.with_ymd_and_hms(2017, 1, 5, 0, 0, 0).unwrap();

    let ret = api
        .timeseries_rates(start_date, end_date, global::BASE_CURRENCY)
        .await;
    dbg!(&ret);

    for v in ret.as_ref().unwrap() {
        assert!(v.error.is_none());
    }
    assert_eq!(ret.as_ref().unwrap().len(), 4);
}

#[tokio::test]
pub async fn test_tradermade_latest_api() {
    let api = tradermade::Api::new(
        &global::config().forex_tradermade_api_key,
        global::http_client(),
    );

    let ret = forex::interface::ForexRates::rates(&api, global::BASE_CURRENCY).await;

    dbg!(&ret);

    assert!(ret.as_ref().unwrap().error.is_none());
    assert!(ret.is_ok());
}

#[tokio::test]
pub async fn test_tradermade_historical_api() {
    let api = tradermade::Api::new(
        &global::config().forex_tradermade_api_key,
        global::http_client(),
    );

    let date = Utc.with_ymd_and_hms(2022, 2, 4, 0, 0, 0).unwrap();

    let ret = api.historical_rates(date, global::BASE_CURRENCY).await;

    dbg!(&ret);
    assert!(ret.as_ref().is_ok());
    assert!(ret.as_ref().unwrap().error.is_none());
}

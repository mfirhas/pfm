use chrono::{TimeZone, Utc};
use pfm_core::{
    forex::{
        interface::ForexStorage,
        service::{poll_historical_rates, poll_rates},
        Money,
    },
    forex_impl::{
        self,
        forex_storage::{self, ForexStorageImpl},
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
    let date = Utc.with_ymd_and_hms(2007, 6, 6, 0, 0, 0).unwrap();
    let ret = poll_historical_rates(&api, &storage, date, global::BASE_CURRENCY).await;
    dbg!(&ret);

    assert!(&ret.is_ok());

    assert!(ret.as_ref().unwrap().error.is_none());
}

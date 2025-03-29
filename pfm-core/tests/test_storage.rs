use core::panic;

use chrono::{TimeDelta, TimeZone, Utc};
use pfm_core::{
    forex::{
        interface::{ForexStorage, ForexTimeseriesRates},
        Money,
    },
    forex_impl::{self, forex_storage::ForexStorageImpl},
    global,
};
use rust_decimal_macros::dec;

// test the forex storage impl on update historical
// latest test: pass
#[tokio::test]
pub async fn test_storage_update_historical() {
    dbg!("running test update historical");
    println!("running test update historical");
    let storage_impl = ForexStorageImpl::new(global::storage_fs());
    let date = Utc.with_ymd_and_hms(2002, 2, 25, 0, 0, 0).unwrap();
    let new_data = vec![
        Money::XAU(dec!(0.04220322222)),
        Money::XAG(dec!(0.23011116)),
    ];
    let before = ForexStorage::get_historical(&storage_impl, date)
        .await
        .unwrap();
    dbg!(&before);
    let after =
        ForexStorage::update_historical_rates_data(&storage_impl, date, new_data.clone()).await;
    dbg!(&after);

    assert_eq!(after.as_ref().unwrap().data.rates.xau, new_data[0].amount());
    assert_eq!(after.as_ref().unwrap().data.rates.xag, new_data[1].amount());
}

#[tokio::test]
pub async fn test_storage_insert_batch() {
    let api = forex_impl::currencybeacon::Api::new(
        &global::config().forex_currencybeacon_api_key,
        global::http_client(),
    );

    let start_date = Utc.with_ymd_and_hms(1996, 1, 1, 0, 0, 0).unwrap();
    let end_date = Utc.with_ymd_and_hms(1996, 1, 12, 0, 0, 0).unwrap();

    let ret = api
        .timeseries_rates(start_date, end_date, global::BASE_CURRENCY)
        .await;
    dbg!(&ret);

    let storage_impl = ForexStorageImpl::new(global::storage_fs());
    let ret = ForexStorage::insert_historical_batch(&storage_impl, ret.unwrap()).await;
    dbg!(&ret);
}

#[tokio::test]
pub async fn test_storage_get_historical_range() {
    let storage = ForexStorageImpl::new(global::storage_fs());
    let start_date = Utc.with_ymd_and_hms(2005, 01, 01, 0, 0, 0).unwrap();
    let end_date = Utc.with_ymd_and_hms(2010, 12, 31, 23, 59, 59).unwrap();
    let ret = ForexStorage::get_historical_range(&storage, start_date, end_date).await;
    // dbg!(&ret);
    // dbg!(&ret.as_ref().unwrap()[200..205]);
    let count_years =
        |start_date: chrono::DateTime<Utc>, end_date: chrono::DateTime<Utc>| -> usize {
            let mut count = 0 as usize;
            let mut date = start_date;
            while date <= end_date {
                count += 1;
                date = date + TimeDelta::days(1);
            }
            count
        };
    dbg!(&ret.as_ref().unwrap()[2189..2191]);
    println!(
        "num of days from the range: {}",
        ret.as_ref().unwrap().len()
    );
    assert_eq!(
        ret.as_ref().unwrap().len(),
        count_years(start_date, end_date)
    );
    // assert all data are within date range(inclusive)
    for v in ret.as_ref().unwrap() {
        if v.data.date < start_date || v.data.date > end_date {
            panic!("historical range contains date smaller than start date, or bigger than end date: {}",v.data.date);
        }
    }
}

use chrono::{NaiveDate, TimeZone, Utc};

use crate::forex::{ForexHistoricalRates, ForexRates};

use super::exchange_api::Api;
use crate::global::http_client;

#[tokio::test]
async fn test_rates() {
    let client = http_client().clone();
    let api = Api::new(client);

    let ret = api.rates(crate::forex::Currencies::USD).await;

    dbg!(&ret);

    assert!(ret.is_ok());
}

#[tokio::test]
async fn test_historical_rates() {
    let client = http_client().clone();
    let api = Api::new(client);

    let date = NaiveDate::from_ymd_opt(2024, 12, 20)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();

    // note: this return error as exchange_api historical data is very limited.
    // source: https://github.com/fawazahmed0/exchange-api/issues/115
    // let date = NaiveDate::from_ymd_opt(2022, 12, 20)
    //     .unwrap()
    //     .and_hms_opt(0, 0, 0)
    //     .unwrap();

    dbg!(&date);
    println!("{}", &date);

    let utc = Utc.from_utc_datetime(&date);

    dbg!(&utc);
    println!("{}", &utc);

    let ret = api
        .historical_rates(utc, crate::forex::Currencies::USD)
        .await;

    dbg!(&ret);

    assert!(ret.is_ok());
}

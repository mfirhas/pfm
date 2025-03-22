// use chrono::{NaiveDate, TimeZone, Utc};

// use crate::forex::{ForexHistoricalRates, ForexRates};
// use crate::global::{config, http_client};

// #[tokio::test]
// async fn test_rates() {
//     let client = http_client().clone();
//     let api_key = &config().forex_open_exchange_api_key;

//     let api = super::open_exchange_api::Api::new(api_key, client);

//     let ret = api.rates(crate::forex::Currencies::USD).await;

//     dbg!(&ret);

//     assert!(ret.is_ok());
// }

// #[tokio::test]
// async fn test_historical_rates() {
//     let client = http_client().clone();
//     let api_key = &config().forex_open_exchange_api_key;

//     let api = super::open_exchange_api::Api::new(api_key, client);

//     let date = NaiveDate::from_ymd_opt(2020, 12, 18)
//         .unwrap()
//         .and_hms_opt(0, 0, 0)
//         .unwrap();
//     let date = Utc.from_utc_datetime(&date);

//     let ret = api
//         .historical_rates(date, crate::forex::Currencies::USD)
//         .await;

//     dbg!(&ret);

//     assert!(ret.is_ok());
// }

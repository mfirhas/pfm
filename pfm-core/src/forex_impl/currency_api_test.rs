use chrono::{NaiveDate, TimeZone, Utc};

use crate::forex::ForexHistoricalRates;

#[tokio::test]
async fn test_historical_rates() {
    let client = reqwest::Client::new();
    let api_key = "anu";

    let api = super::currency_api::Api::new(api_key, &client);

    let date = NaiveDate::from_ymd_opt(2024, 12, 20)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();
    let date = Utc.from_utc_datetime(&date);

    let ret = api
        .historical_rates(date, crate::forex::Currencies::USD)
        .await;

    dbg!(&ret);

    assert!(ret.is_ok());
}

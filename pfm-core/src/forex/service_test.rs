use chrono::{DateTime, TimeZone, Utc};
use rust_decimal_macros::dec;

use crate::{
    forex::{
        Currency, Money,
        entity::ConversionResponse,
        interface::ForexStorage,
        service::{batch_convert, convert, convert_historical, poll_historical_rates, poll_rates},
    },
    global,
};

#[tokio::test]
async fn test_convert() {
    let fs = global::storage_fs();
    let storage = super::mock::ForexStorageSuccessMock;

    let from = Money::new_money(crate::forex::Currency::GBP, dec!(1000));
    let to = Currency::SAR;
    let ret = convert(&storage, from, to).await;
    dbg!(&ret);

    assert!(ret.is_ok());

    let ret = ret.unwrap();
    // expected data come from forex_mock
    let expected = Money::new_money(Currency::SAR, dec!(4762.0152292578498482026199809));
    assert_eq!(ret.to, expected);
}

#[tokio::test]
async fn test_convert_historical() {
    let fs = global::storage_fs();
    let storage = super::mock::ForexStorageSuccessMock;

    let from = Money::new_money(crate::forex::Currency::GBP, dec!(1000));
    let to = Currency::SAR;
    let date = Utc.with_ymd_and_hms(2022, 12, 25, 0, 0, 0).unwrap();
    let ret = convert_historical(&storage, from, to, date).await;
    dbg!(&ret);

    assert!(ret.is_ok());

    let ret = ret.unwrap();
    // expected data come from forex_mock
    let expected = Money::new_money(Currency::SAR, dec!(4533.0433702899590250394500024));
    assert_eq!(ret.to, expected);
}

#[tokio::test]
async fn test_batch_convert() {
    let fs = global::storage_fs();
    let storage = super::mock::ForexStorageSuccessMock;

    let from_gbp = Money::new_money(crate::forex::Currency::GBP, dec!(1000));
    let from_usd = Money::new_money(crate::forex::Currency::USD, dec!(4000));
    let from_idr = Money::new_money(crate::forex::Currency::IDR, dec!(23000));
    let from_chf = Money::new_money(crate::forex::Currency::CHF, dec!(1000));
    let from_sgd = Money::new_money(crate::forex::Currency::SGD, dec!(1300));
    let from = vec![from_gbp, from_usd, from_idr, from_chf, from_sgd];
    let to = Currency::SAR;
    let ret = batch_convert(&storage, from, to).await;
    dbg!(&ret);

    // expected data come from forex_mock
    let expected_conversions = vec![
        ConversionResponse {
            date: DateTime::parse_from_rfc3339("2025-03-04T02:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            from: from_gbp,
            to: Money::SAR(dec!(4762.0152292578498482026199809)),
            code: Money::SAR(dec!(4762.0152292578498482026199809)).format(false),
            symbol: Money::SAR(dec!(4762.0152292578498482026199809)).format(true),
        },
        ConversionResponse {
            date: DateTime::parse_from_rfc3339("2025-03-04T02:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            from: from_usd,
            to: Money::SAR(dec!(15001.548000)),
            code: Money::SAR(dec!(15001.548000)).format(false),
            symbol: Money::SAR(dec!(15001.548000)).format(true),
        },
        ConversionResponse {
            date: DateTime::parse_from_rfc3339("2025-03-04T02:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            from: from_idr,
            to: Money::SAR(dec!(5.2401981046108984873336978311)),
            code: Money::SAR(dec!(5.2401981046108984873336978311)).format(false),
            symbol: Money::SAR(dec!(5.2401981046108984873336978311)).format(true),
        },
        ConversionResponse {
            date: DateTime::parse_from_rfc3339("2025-03-04T02:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            from: from_chf,
            to: Money::SAR(dec!(4186.4940892803322058872777200)),
            code: Money::SAR(dec!(4186.4940892803322058872777200)).format(false),
            symbol: Money::SAR(dec!(4186.4940892803322058872777200)).format(true),
        },
        ConversionResponse {
            date: DateTime::parse_from_rfc3339("2025-03-04T02:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            from: from_sgd,
            to: Money::SAR(dec!(3625.2651561342823236183774170)),
            code: Money::SAR(dec!(3625.2651561342823236183774170)).format(false),
            symbol: Money::SAR(dec!(3625.2651561342823236183774170)).format(true),
        },
    ];

    assert!(ret.is_ok());
    assert_eq!(ret.as_ref().unwrap().len(), 5);
    let ret = ret.unwrap();
    for (i, v) in expected_conversions.iter().enumerate() {
        assert_eq!(ret[i].to, v.to);
    }
}

#[tokio::test]
async fn test_poll_rates() {
    let cfg = global::config();
    let fs = global::storage_fs();
    let http_client = global::http_client();
    let storage = super::mock::ForexStorageSuccessMock;
    let forex = super::mock::ForexApiSuccessMock;

    let base = Currency::USD;
    let ret = poll_rates(&forex, &storage, base).await;
    dbg!(&ret);

    assert!(ret.is_ok());
    assert!(ret.as_ref().unwrap().error.is_none());
    assert_eq!(ret.unwrap().data.base, Currency::USD);
}

#[tokio::test]
async fn test_poll_historical_rates() {
    let cfg = global::config();
    let fs = global::storage_fs();
    let http_client = global::http_client();
    let storage = super::mock::ForexStorageSuccessMock;
    let forex = super::mock::ForexApiSuccessMock;

    let base = Currency::USD;
    let date = Utc.with_ymd_and_hms(2022, 12, 25, 0, 0, 0).unwrap();
    let ret = poll_historical_rates(&forex, &storage, date, base).await;
    dbg!(&ret);

    assert!(ret.is_ok());
    assert!(ret.as_ref().unwrap().error.is_none());
    assert_eq!(ret.unwrap().data.base, Currency::USD);
}

#[tokio::test]
async fn test_get_rates_list() {
    let fs = global::storage_fs();
    let storage = super::mock::ForexStorageSuccessMock;

    let ret = storage
        .get_latest_list(1, 5, crate::forex::entity::Order::DESC)
        .await;
    dbg!(&ret);
    let ret = ret.unwrap();
    assert!(ret.rates_list.len().eq(&5));
    assert_eq!(ret.has_prev, false);
    assert_eq!(ret.has_next, true);
    assert!(ret.rates_list[0].data.latest_update > ret.rates_list[1].data.latest_update);
}

#[tokio::test]
async fn test_get_historical_list() {
    let fs = global::storage_fs();
    let storage = super::mock::ForexStorageSuccessMock;

    let ret = storage
        .get_historical_list(1, 5, crate::forex::entity::Order::DESC)
        .await;
    dbg!(&ret);
    let ret = ret.unwrap();
    assert!(ret.rates_list.len().eq(&4));
    assert_eq!(ret.has_prev, false);
    assert_eq!(ret.has_next, false);
    assert!(ret.rates_list[0].data.date > ret.rates_list[1].data.date);
}

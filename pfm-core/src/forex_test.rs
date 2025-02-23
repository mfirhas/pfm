mod regex_format_tests {
    use super::super::forex::{COMMA_SEPARATOR_REGEX, DOT_SEPARATOR_REGEX};

    #[test]
    fn test_dot_separated_amount() {
        let regex = &DOT_SEPARATOR_REGEX;

        // Valid cases
        assert!(
            regex.is_match("1.000.000.000,24"),
            "should match 1.000.000.000,24"
        );
        assert!(regex.is_match("1.000.000,00"), "should match 1.000.000,00");
        assert!(regex.is_match("1.000"), "should match 1.000");
        assert!(regex.is_match("1000"), "should match 1000");
        assert!(regex.is_match("1.000,50"), "should match 1.000,50");
        assert!(regex.is_match("01.000"), "should match 01.000");
        assert!(regex.is_match("1.234.567,89"), "should match 1.234.567,89");
        assert!(regex.is_match("1000.000"), "should match 1.000.000");

        // Invalid cases
        assert!(
            !regex.is_match("1,000.000.000,24"),
            "should match 1.000.000.000,24"
        );

        assert!(
            !regex.is_match("1.00.000"),
            "should not match 1.00.000 (wrong grouping)"
        );
        assert!(
            !regex.is_match("1,000.00"),
            "should not match 1,000.00 (wrong separators)"
        );
        assert!(
            !regex.is_match("1.000."),
            "should not match 1.000. (trailing separator)"
        );
        assert!(
            !regex.is_match(".1.000"),
            "should not match .1.000 (leading separator)"
        );
        assert!(
            !regex.is_match("1.000,0"),
            "should not match 1.000,0 (single decimal)"
        );
        assert!(
            !regex.is_match("1.000,000"),
            "should not match 1.000,000 (three decimals)"
        );
        assert!(
            !regex.is_match("1,00,00"),
            "should not match 100,00 (three decimals)"
        );
    }

    #[test]
    fn test_comma_separated_amount() {
        let regex = &COMMA_SEPARATOR_REGEX;

        // Valid cases
        assert!(
            regex.is_match("1,000,000,000.24"),
            "should match 1,000,000,000.24"
        );

        assert!(regex.is_match("1,000,000.00"), "should match 1,000,000.00");
        assert!(regex.is_match("1,000"), "should match 1,000");
        assert!(regex.is_match("1000"), "should match 1000");
        assert!(regex.is_match("1,000.50"), "should match 1,000.50");
        assert!(regex.is_match("01,000"), "should match 01,000");
        assert!(regex.is_match("1,234,567.89"), "should match 1,234,567.89");
        assert!(regex.is_match("1000,000"), "should match 1,000,000");

        // Invalid cases
        assert!(
            !regex.is_match("1.000,000,000.24"),
            "should match 1,000,000,000.24"
        );

        assert!(
            !regex.is_match("1,00,000"),
            "should not match 1,00,000 (wrong grouping)"
        );
        assert!(
            !regex.is_match("1.000,00"),
            "should not match 1.000,00 (wrong separators)"
        );
        assert!(
            !regex.is_match("1,000,"),
            "should not match 1,000, (trailing separator)"
        );
        assert!(
            !regex.is_match(",1,000"),
            "should not match ,1,000 (leading separator)"
        );
        assert!(
            !regex.is_match("1,000.0"),
            "should not match 1,000.0 (single decimal)"
        );
        assert!(
            !regex.is_match("1,000.000"),
            "should not match 1,000.000 (three decimals)"
        );
        assert!(!regex.is_match("1.00.00"), "should match 100.00");
    }
}

use std::str::FromStr;

use chrono::{TimeZone, Utc};
use rust_decimal_macros::dec;

use crate::{
    forex::ForexStorage,
    forex::{convert, poll_historical_rates, poll_rates, Currency},
    forex_impl, forex_storage_impl, global,
};

use super::forex::Money;
#[test]
fn test_money() {
    let expected = "USD 23,000";
    let money = Money::new("USD", "23000");
    dbg!(&money);
    println!("{}", money.as_ref().unwrap());
    assert!(money.is_ok());
    assert_eq!(money.unwrap().to_string().as_str(), expected);

    let expected = "IDR 45.000.000"; // indonesian rupiah is dot separated for thousands.
    let money = Money::new("IDR", "45000000");
    dbg!(&money);
    println!("{}", money.as_ref().unwrap());
    assert!(money.is_ok());
    assert_eq!(money.unwrap().to_string().as_str(), expected);
}

#[test]
fn test_money_from_str() {
    let input = "USD 23,000";
    let expected = "USD 23,000";
    let money = Money::from_str(input);
    dbg!(&money);
    println!("{}", money.as_ref().unwrap());
    assert!(money.is_ok());
    assert_eq!(money.unwrap().to_string().as_str(), expected);

    // comma separated currencies cannot be written in dot separated.
    let input = "USD 23.000";
    let money = Money::from_str(input);
    dbg!(&money);
    // println!("{}", money.as_ref().unwrap());
    assert!(money.is_err());

    let input = "IDR 23.000";
    let expected = "IDR 23.000";
    let money = Money::from_str(input);
    dbg!(&money);
    println!("{}", money.as_ref().unwrap());
    assert!(money.is_ok());
    assert_eq!(money.unwrap().to_string().as_str(), expected);

    // dot separated currencies can be written in comma separated
    let input = "IDR 23,000";
    let expected = "IDR 23.000";
    let money = Money::from_str(input);
    dbg!(&money);
    println!("{}", money.as_ref().unwrap());
    assert!(money.is_ok());
    assert_eq!(money.unwrap().to_string().as_str(), expected);

    //// without thousands separator
    let input = "USD 23000";
    let expected = "USD 23,000";
    let money = Money::from_str(input);
    dbg!(&money);
    println!("{}", money.as_ref().unwrap());
    assert!(money.is_ok());
    assert_eq!(money.unwrap().to_string().as_str(), expected);

    let input = "IDR 23000";
    let expected = "IDR 23.000";
    let money = Money::from_str(input);
    dbg!(&money);
    println!("{}", money.as_ref().unwrap());
    assert!(money.is_ok());
    assert_eq!(money.unwrap().to_string().as_str(), expected);

    // dot separated currencies can be written in comma separated
    let input = "IDR 23000";
    let expected = "IDR 23.000";
    let money = Money::from_str(input);
    dbg!(&money);
    println!("{}", money.as_ref().unwrap());
    assert!(money.is_ok());
    assert_eq!(money.unwrap().to_string().as_str(), expected);
}

// make sure to test get_rates first to populate directory before testing this
#[tokio::test]
async fn test_convert() {
    let fs = global::storage_fs();
    let storage = forex_storage_impl::forex_storage::ForexStorageImpl::new(fs);

    let from = Money::new_money(crate::forex::Currency::GBP, dec!(1000));
    let to = Currency::SAR;
    let ret = convert(&storage, from, to).await;
    dbg!(&ret);

    assert!(ret.is_ok());
}

#[tokio::test]
async fn test_poll_rates() {
    let cfg = global::config();
    let fs = global::storage_fs();
    let http_client = global::http_client();
    let storage = forex_storage_impl::forex_storage::ForexStorageImpl::new(fs);
    let forex =
        forex_impl::open_exchange_api::Api::new(&cfg.forex_open_exchange_api_key, http_client);

    let base = Currency::USD;
    let ret = poll_rates(&forex, &storage, base).await;
    dbg!(&ret);

    assert!(ret.is_ok());
}

#[tokio::test]
async fn test_poll_historical_rates() {
    let cfg = global::config();
    let fs = global::storage_fs();
    let http_client = global::http_client();
    let storage = forex_storage_impl::forex_storage::ForexStorageImpl::new(fs);
    let forex =
        forex_impl::open_exchange_api::Api::new(&cfg.forex_open_exchange_api_key, http_client);

    let base = Currency::USD;
    let date = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
    let ret = poll_historical_rates(&forex, &storage, date, base).await;
    dbg!(&ret);

    assert!(ret.is_ok());
}

#[tokio::test]
async fn test_get_rates_list() {
    let fs = global::storage_fs();
    let storage = forex_storage_impl::forex_storage::ForexStorageImpl::new(fs);

    let ret = storage
        .get_latest_list(1, 5, crate::forex::Order::DESC)
        .await;
    dbg!(&ret);
}

#[tokio::test]
async fn test_get_historical_list() {
    let fs = global::storage_fs();
    let storage = forex_storage_impl::forex_storage::ForexStorageImpl::new(fs);

    let ret = storage
        .get_historical_list(1, 5, crate::forex::Order::DESC)
        .await;
    dbg!(&ret);
}

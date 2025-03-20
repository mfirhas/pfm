use std::str::FromStr;

use rust_decimal_macros::dec;
use strum::IntoEnumIterator;

use crate::forex::{Currency, Money};

/// make sure variants of currency checked
#[test]
fn test_currency_items() {
    let currency_variants_count = Currency::iter().count();
    let expected_count = 9;
    assert_eq!(currency_variants_count, expected_count);
}

#[test]
fn test_currency_code() {
    let expected_idr = "IDR";
    let expected_usd = "USD";
    let expected_gbp = "GBP";
    let expected_sar = "SAR";

    assert_eq!(Currency::IDR.code(), expected_idr);
    assert_eq!(Currency::USD.code(), expected_usd);
    assert_eq!(Currency::GBP.code(), expected_gbp);
    assert_eq!(Currency::SAR.code(), expected_sar);
}

#[test]
fn test_currency_from_str() {
    let expected_usd = Currency::USD;
    let expected_idr = Currency::IDR;
    let expected_chf = Currency::CHF;
    let expected_jpy = Currency::JPY;

    let ret = Currency::from_str("USD");
    assert!(ret.is_ok());
    assert_eq!(ret.unwrap(), expected_usd);

    let ret = Currency::from_str("IDR");
    assert!(ret.is_ok());
    assert_eq!(ret.unwrap(), expected_idr);

    let ret = Currency::from_str("CHF");
    assert!(ret.is_ok());
    assert_eq!(ret.unwrap(), expected_chf);

    let ret = Currency::from_str("JPY");
    assert!(ret.is_ok());
    assert_eq!(ret.unwrap(), expected_jpy);
}

#[test]
fn test_currency_default() {
    let expected_default = Currency::USD;

    let ret = Currency::default();

    assert_eq!(ret, expected_default);
}

#[test]
fn test_currency_display() {
    let expected_usd = "USD".to_string();
    let expected_idr = "IDR".to_string();
    let expected_gbp = "GBP".to_string();
    let expected_cny = "CNY".to_string();

    assert_eq!(Currency::USD.to_string(), expected_usd);
    assert_eq!(Currency::IDR.to_string(), expected_idr);
    assert_eq!(Currency::GBP.to_string(), expected_gbp);
    assert_eq!(Currency::CNY.to_string(), expected_cny);
}

#[test]
fn test_currency_from_money() {
    let expected_usd = Currency::USD;
    let expected_idr = Currency::IDR;
    let expected_sar = Currency::SAR;
    let expected_sgd = Currency::SGD;

    let ret = Currency::from(Money::new_money(Currency::USD, dec!(1000)));
    assert_eq!(ret, expected_usd);
    let ret = Currency::from(Money::new_money(Currency::IDR, dec!(1000)));
    assert_eq!(ret, expected_idr);
    let ret = Currency::from(Money::new_money(Currency::SAR, dec!(1000)));
    assert_eq!(ret, expected_sar);
    let ret = Currency::from(Money::new_money(Currency::SGD, dec!(1000)));
    assert_eq!(ret, expected_sgd);
}

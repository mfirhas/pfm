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
    forex::{batch_convert, convert, poll_historical_rates, poll_rates, Currency, ForexStorage},
    forex_impl, global,
};

/// test crate::forex::Currency
mod currency_tests {
    use std::str::FromStr;

    use rust_decimal_macros::dec;

    use crate::forex::{Currency, Money};

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

    #[test]
    fn test_currency_partial_eq_money() {
        let money_usd = Money::new_money(Currency::USD, dec!(1000));
        let money_idr = Money::new_money(Currency::IDR, dec!(1000));
        let money_sgd = Money::new_money(Currency::SGD, dec!(1000));
        let money_eur = Money::new_money(Currency::EUR, dec!(1000));

        assert_eq!(money_usd, Currency::USD);
        assert_eq!(money_eur, Currency::EUR);
        assert_eq!(money_sgd, Currency::SGD);
        assert_eq!(money_idr, Currency::IDR);
    }
}

mod money_tests {
    use std::str::FromStr;

    use rust_decimal_macros::dec;

    use crate::forex::Currency;
    use crate::forex::Money;

    #[test]
    fn test_impl() {
        let expected_usd = Money::new_money(Currency::USD, dec!(1000));
        let expected_idr = Money::new_money(Currency::IDR, dec!(234000));
        let expected_chf = Money::new_money(Currency::CHF, dec!(412300));
        let expected_eur = Money::new_money(Currency::EUR, dec!(8000));
        let expected_sar = Money::new_money(Currency::SAR, dec!(5000));

        let ret = Money::new("USD", "1000");
        assert_eq!(ret.as_ref().unwrap(), &expected_usd);
        assert_eq!(Currency::USD, ret.as_ref().unwrap().currency());
        assert_eq!("USD".to_string(), ret.as_ref().unwrap().code());
        assert_eq!("$".to_string(), ret.unwrap().symbol());

        let ret_idr = Money::new("IDR", "234000");
        assert_eq!(ret_idr.as_ref().unwrap(), &expected_idr);
        assert_eq!(Currency::IDR, ret_idr.as_ref().unwrap().currency());
        assert_eq!("IDR".to_string(), ret_idr.as_ref().unwrap().code());
        assert_eq!("Rp".to_string(), ret_idr.unwrap().symbol());

        let ret_chf = Money::new("CHF", "412300");
        assert_eq!(ret_chf.as_ref().unwrap(), &expected_chf);
        assert_eq!(Currency::CHF, ret_chf.as_ref().unwrap().currency());
        assert_eq!("CHF".to_string(), ret_chf.as_ref().unwrap().code());
        assert_eq!("₣".to_string(), ret_chf.unwrap().symbol());

        let ret_eur = Money::new("EUR", "8000");
        assert_eq!(ret_eur.as_ref().unwrap(), &expected_eur);
        assert_eq!(Currency::EUR, ret_eur.as_ref().unwrap().currency());
        assert_eq!("EUR".to_string(), ret_eur.as_ref().unwrap().code());
        assert_eq!("€".to_string(), ret_eur.unwrap().symbol());

        let ret_sar = Money::new("SAR", "5000");
        assert_eq!(ret_sar.as_ref().unwrap(), &expected_sar);
        assert_eq!(Currency::SAR, ret_sar.as_ref().unwrap().currency());
        assert_eq!("SAR".to_string(), ret_sar.as_ref().unwrap().code());
        assert_eq!("ر.س".to_string(), ret_sar.unwrap().symbol());
    }

    #[test]
    fn test_money_to_string() {
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
}

use super::forex::Money;

// make sure to test get_rates first to populate directory before testing this
#[tokio::test]
async fn test_convert() {
    let fs = global::storage_fs();
    let storage = forex_impl::forex_storage::ForexStorageImpl::new(fs);

    let from = Money::new_money(crate::forex::Currency::GBP, dec!(1000));
    let to = Currency::SAR;
    let ret = convert(&storage, from, to).await;
    dbg!(&ret);

    assert!(ret.is_ok());
}

#[tokio::test]
async fn test_batch_convert() {
    let fs = global::storage_fs();
    let storage = forex_impl::forex_storage::ForexStorageImpl::new(fs);

    let from_gbp = Money::new_money(crate::forex::Currency::GBP, dec!(1000));
    let from_usd = Money::new_money(crate::forex::Currency::USD, dec!(4000));
    let from_idr = Money::new_money(crate::forex::Currency::IDR, dec!(23000));
    let from_chf = Money::new_money(crate::forex::Currency::CHF, dec!(1000));
    let from_sgd = Money::new_money(crate::forex::Currency::SGD, dec!(1300));
    let from = vec![from_gbp, from_usd, from_idr, from_chf, from_sgd];
    let to = Currency::SAR;
    let ret = batch_convert(&storage, from, to).await;
    dbg!(&ret);

    assert!(ret.is_ok());
    assert_eq!(ret.unwrap().len(), 5);
}

#[tokio::test]
async fn test_poll_rates() {
    let cfg = global::config();
    let fs = global::storage_fs();
    let http_client = global::http_client();
    let storage = forex_impl::forex_storage::ForexStorageImpl::new(fs);
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
    let storage = forex_impl::forex_storage::ForexStorageImpl::new(fs);
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
    let storage = forex_impl::forex_storage::ForexStorageImpl::new(fs);

    let ret = storage
        .get_latest_list(1, 5, crate::forex::Order::DESC)
        .await;
    dbg!(&ret);
}

#[tokio::test]
async fn test_get_historical_list() {
    let fs = global::storage_fs();
    let storage = forex_impl::forex_storage::ForexStorageImpl::new(fs);

    let ret = storage
        .get_historical_list(1, 5, crate::forex::Order::DESC)
        .await;
    dbg!(&ret);
}

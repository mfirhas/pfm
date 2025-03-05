mod money_format_regex_tests {
    use core::panic;

    use super::super::forex::MONEY_FORMAT_REGEX;

    #[test]
    fn test_money_format_regex() {
        // HAPPY PATH TEST CASES
        let happy_path_tests = vec![
            // Basic formats
            "USD 100",
            "EUR 200",
            "JPY 300",
            // With comma separators
            "USD 1,000",
            "USD 1,000,000",
            "USD 1,000,000,000",
            // With decimal places
            "USD 100.00",
            "USD 100.50",
            "USD 100.12345", // Multiple decimal places
            // Combining comma separators and decimal places
            "USD 1,000.00",
            "USD 1,000,000.50",
            "USD 1,000,000,000.99",
            // Different currency codes
            "IDR 50000",
            "GBP 1,234.56",
            "AUD 9,876.54",
            // Single digit
            "USD 1",
            "USD 1.5",
            // Large numbers
            "BTC 0.00000001",
            "IDR 999,999,999,999.99999",
            "USD  100", // Multiple spaces (this actually works with our regex due to \s+)
        ];

        // UNHAPPY PATH TEST CASES
        let unhappy_path_tests = vec![
            // Invalid currency code format
            "Us 100",   // Lowercase letters
            "USDD 100", // More than 3 letters
            "US 100",   // Less than 3 letters
            "123 100",  // Numbers instead of letters
            // Invalid spacing
            "USD100", // No space
            // Invalid amount format
            "USD 1,00",     // Incorrect comma placement
            "USD 1,0000",   // Incorrect grouping
            "USD 1.000.00", // Multiple decimal points
            "USD 1,000,00", // Incorrect comma placement
            "USD ,100",     // Starting with comma
            "USD 100,",     // Ending with comma
            // Decimal separator issues
            "USD 100,00",   // Using comma as decimal separator (European style)
            "USD 1.000,00", // European number format
            // Invalid characters
            "USD 100a", // Letters in amount
            "USD $100", // Symbols in amount
            "USD -100", // Negative numbers
            "USD +100", // Explicit positive sign
            // Empty or missing values
            "USD ", // Missing amount
            " 100", // Missing currency code
            "",     // Empty string
            // Extra information
            "USD 100.00 only", // Extra text
            "Price: USD 100",  // Extra text
        ];

        for v in happy_path_tests {
            let ret = MONEY_FORMAT_REGEX.is_match(v);
            if !ret {
                panic!("test_money_format_regex error on happy_path_tests: expected '{}' to be validated", v);
            }
        }

        for v in unhappy_path_tests {
            let ret = MONEY_FORMAT_REGEX.is_match(v);
            if ret {
                panic!("test_money_format_regex error on unhappy_path_tests: expected '{}' to be validated", v);
            }
        }
    }
}

use std::str::FromStr;

use chrono::{DateTime, TimeZone, Utc};
use rust_decimal_macros::dec;

use crate::{
    forex::{
        batch_convert, convert, poll_historical_rates, poll_rates, ConversionResponse, Currency,
        ForexStorage, Money,
    },
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

        let expected = "IDR 45,000,000"; // indonesian rupiah is dot separated for thousands.
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

        // will be treated as dot separating fraction
        let input = "USD 23.000";
        let money = Money::from_str(input);
        dbg!("-->", &money);
        // println!("{}", money.as_ref().unwrap());
        assert!(money.is_ok());

        let input = "IDR 23,000";
        let expected = "IDR 23,000";
        let money = Money::from_str(input);
        dbg!(&money);
        println!("{}", money.as_ref().unwrap());
        assert!(money.is_ok());
        assert_eq!(money.unwrap().to_string().as_str(), expected);

        // dot separated currencies can be written in comma separated
        let input = "IDR 23,000";
        let expected = "IDR 23,000";
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
        let expected = "IDR 23,000";
        let money = Money::from_str(input);
        dbg!(&money);
        println!("{}", money.as_ref().unwrap());
        assert!(money.is_ok());
        assert_eq!(money.unwrap().to_string().as_str(), expected);

        // dot separated currencies can be written in comma separated
        let input = "IDR 23000";
        let expected = "IDR 23,000";
        let money = Money::from_str(input);
        dbg!(&money);
        println!("{}", money.as_ref().unwrap());
        assert!(money.is_ok());
        assert_eq!(money.unwrap().to_string().as_str(), expected);
    }

    #[test]
    fn test_money_equality() {
        let a = Money::new_money(Currency::USD, dec!(42533));
        let b = Money::new_money(Currency::USD, dec!(42533));
        assert_eq!(a, b);

        let a = Money::new_money(Currency::USD, dec!(1.234));
        let b = Money::new_money(Currency::USD, dec!(1.234));
        assert_eq!(a, b);

        let a = Money::new_money(Currency::USD, dec!(1.2345));
        let b = Money::new_money(Currency::USD, dec!(1.234));
        assert_ne!(a, b);

        let a = Money::new_money(Currency::USD, dec!(1.234));
        let b = Money::new_money(Currency::IDR, dec!(1.234));
        assert_ne!(a, b);
    }
}

// make sure to test get_rates first to populate directory before testing this
#[tokio::test]
async fn test_convert() {
    let fs = global::storage_fs();
    let storage = super::forex_mock::ForexStorageSuccessMock;

    let from = Money::new_money(crate::forex::Currency::GBP, dec!(1000));
    let to = Currency::SAR;
    let ret = convert(&storage, from, to).await;
    dbg!(&ret);

    assert!(ret.is_ok());

    let ret = ret.unwrap();
    // expected data come from forex_mock
    let expected = Money::new_money(Currency::SAR, dec!(4762.0152292578498482026199809));
    assert_eq!(ret.money, expected);
}

#[tokio::test]
async fn test_batch_convert() {
    let fs = global::storage_fs();
    let storage = super::forex_mock::ForexStorageSuccessMock;

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
            last_update: DateTime::parse_from_rfc3339("2025-03-04T02:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            money: Money::SAR(dec!(4762.0152292578498482026199809)),
        },
        ConversionResponse {
            last_update: DateTime::parse_from_rfc3339("2025-03-04T02:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            money: Money::SAR(dec!(15001.548000)),
        },
        ConversionResponse {
            last_update: DateTime::parse_from_rfc3339("2025-03-04T02:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            money: Money::SAR(dec!(5.2401981046108984873336978311)),
        },
        ConversionResponse {
            last_update: DateTime::parse_from_rfc3339("2025-03-04T02:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            money: Money::SAR(dec!(4186.4940892803322058872777200)),
        },
        ConversionResponse {
            last_update: DateTime::parse_from_rfc3339("2025-03-04T02:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            money: Money::SAR(dec!(3625.2651561342823236183774170)),
        },
    ];

    assert!(ret.is_ok());
    assert_eq!(ret.as_ref().unwrap().len(), 5);
    let ret = ret.unwrap();
    for (i, v) in expected_conversions.iter().enumerate() {
        assert_eq!(ret[i].money, v.money);
    }
}

#[tokio::test]
async fn test_poll_rates() {
    let cfg = global::config();
    let fs = global::storage_fs();
    let http_client = global::http_client();
    let storage = super::forex_mock::ForexStorageSuccessMock;
    let forex = super::forex_mock::ForexApiSuccessMock;

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
    let storage = super::forex_mock::ForexStorageSuccessMock;
    let forex = super::forex_mock::ForexApiSuccessMock;

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
    let storage = super::forex_mock::ForexStorageSuccessMock;

    let ret = storage
        .get_latest_list(1, 5, crate::forex::Order::DESC)
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
    let storage = super::forex_mock::ForexStorageSuccessMock;

    let ret = storage
        .get_historical_list(1, 5, crate::forex::Order::DESC)
        .await;
    dbg!(&ret);
    let ret = ret.unwrap();
    assert!(ret.rates_list.len().eq(&4));
    assert_eq!(ret.has_prev, false);
    assert_eq!(ret.has_next, false);
    assert!(ret.rates_list[0].data.date > ret.rates_list[1].data.date);
}

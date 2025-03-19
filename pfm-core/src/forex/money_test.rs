use core::panic;

use super::money::MONEY_FORMAT_REGEX;

/// make sure variants of money checked
#[test]
fn test_money_items() {
    let money_variants_count = Money::iter().count();
    let expected_count = 9;
    assert_eq!(money_variants_count, expected_count);
}

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
            panic!(
                "test_money_format_regex error on happy_path_tests: expected '{}' to be validated",
                v
            );
        }
    }

    for v in unhappy_path_tests {
        let ret = MONEY_FORMAT_REGEX.is_match(v);
        if ret {
            panic!("test_money_format_regex error on unhappy_path_tests: expected '{}' to be validated", v);
        }
    }
}

use std::str::FromStr;

use rust_decimal_macros::dec;
use strum::IntoEnumIterator;

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

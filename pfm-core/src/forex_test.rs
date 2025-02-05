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

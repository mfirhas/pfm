use crate::forex::*;
use accounting::Accounting;
use anyhow::anyhow;
use iso_currency::Currency;
use rust_decimal::Decimal;
use std::str::FromStr;

pub(crate) fn parse_str(input_money: &str) -> ForexResult<Money> {
    // 1. Check currency parts
    // currency parts <CODE> <major-unit.minor-unit>
    let currency_parts: Vec<&str> = input_money.split_whitespace().collect();
    if currency_parts.len() != 2 {
        return Err(anyhow!(ERROR_CURRENCY_PARTS));
    }
    let currency = Currency::from_str(currency_parts[0])
        .map_err(|err| anyhow!(format!("invalid currency code: {}", err)))?;

    // 2. check if first part is in list of currency codes
    let code = Currency::from_str(currency_parts[0])
        .map_err(|err| anyhow!(format!("Currency code invalid: {}", err)))?;

    // 3. check if the code is in comma or dot separated for thousands.
    let is_comma_separated_code = COMMA_SEPARATED_CURRENCIES.contains(&code);

    // 4. validate format using regex
    let is_amount_validated = COMMA_SEPARATOR_REGEX.is_match(currency_parts[1]);
    if !is_amount_validated && !is_comma_separated_code {
        let is_amount_validated = DOT_SEPARATOR_REGEX.is_match(currency_parts[1]);
        if !is_amount_validated {
            return Err(anyhow!(ERROR_INVALID_AMOUNT_FORMAT));
        }
    } else {
        return Err(anyhow!(ERROR_INVALID_AMOUNT_FORMAT));
    }

    // 5. remove thousands separator and convert decimal/minor unit separator to dot.
    let amount = if is_comma_separated_code {
        let ch = ',';
        let ret: String = currency_parts[1].chars().filter(|&c| c != ch).collect();
        ret
    } else {
        let ch = '.';
        let ret: String = currency_parts[1].chars().filter(|&c| c != ch).collect();
        let ret: String = ret
            .chars()
            .map(|c| if c == ',' { '.' } else { c })
            .collect();
        ret
    };

    // 6. convert amount into Decimal.
    let decimal = Decimal::from_str(&amount).map_err(|err| {
        anyhow!(format!(
            "failed converting amount into decimal type: {}",
            err
        ))
    })?;

    Ok(Money {
        currency,
        amount: decimal,
    })
}

pub(crate) fn to_string(use_symbol: bool, money: &Money) -> String {
    let currency_code: &str = if use_symbol {
        &money.currency.symbol().to_string()
    } else {
        money.currency.code()
    };

    let mut ac = if COMMA_SEPARATED_CURRENCIES.contains(&money.currency) {
        Accounting::new_from_seperator(currency_code, 2, ",", ".")
    } else {
        Accounting::new_from_seperator(currency_code, 2, ".", ",")
    };

    if use_symbol {
        ac.set_format("{s}{v}");
    } else {
        ac.set_format("{s} {v}");
    }

    let money_display = ac.format_money(money.amount);

    money_display
}

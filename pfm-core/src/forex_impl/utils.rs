use crate::forex::*;
use accounting::Accounting;
use anyhow::{anyhow, Context};
use entity::Rates;
use interface::AsClientError;
use money::{ERROR_MONEY_FORMAT, MONEY_FORMAT_REGEX};
use rust_decimal::Decimal;
use std::str::FromStr;

pub(crate) fn parse_str(input_money: &str) -> ForexResult<Money> {
    // 1. parse with regex
    if !MONEY_FORMAT_REGEX.is_match(input_money) {
        return Err(ForexError::client_error(ERROR_MONEY_FORMAT));
    }

    // 2. take money parts: currency and amount
    let money_parts: Vec<&str> = input_money.split_whitespace().collect();
    if money_parts.len() != 2 {
        return Err(ForexError::client_error(ERROR_MONEY_FORMAT));
    }

    // 3. parse currency code
    let currency = money_parts[0].parse::<Currency>()?;

    // 4. remove thousands separator
    let comma = ',';
    let amount_str: String = money_parts[1].chars().filter(|&c| c != comma).collect();

    // 5. convert amount into Decimal.
    let amount = Decimal::from_str(&amount_str)
        .context("convert amount str to Decimal")
        .as_client_err()?;

    Ok(Money::new_money(currency, amount))
}

pub(crate) fn to_string(use_symbol: bool, money: Money) -> String {
    let currency_code: String = if use_symbol {
        money.symbol()
    } else {
        money.code()
    };

    let mut ac = Accounting::new_from_seperator(currency_code.as_str(), 2, ",", ".");

    if use_symbol {
        ac.set_format("{s}{v}");
    } else {
        ac.set_format("{s} {v}");
    }

    let money_display = ac.format_money(money.amount());

    money_display
}

pub(crate) fn convert_currency(rates: &Rates, from: Money, to: Currency) -> ForexResult<Money> {
    if from == to {
        return Ok(from);
    }

    // 1. divide from with its rate relative to base currency.
    let to_base = match from {
        Money::IDR(amount) => amount / rates.rates.idr,
        Money::USD(amount) => amount / rates.rates.usd,
        Money::EUR(amount) => amount / rates.rates.eur,
        Money::GBP(amount) => amount / rates.rates.gbp,
        Money::JPY(amount) => amount / rates.rates.jpy,
        Money::CHF(amount) => amount / rates.rates.chf,
        Money::SGD(amount) => amount / rates.rates.sgd,
        Money::CNY(amount) => amount / rates.rates.cny,
        Money::SAR(amount) => amount / rates.rates.sar,
    };

    // 2. multiply the above result with the rate of target conversion relative to base currency.
    let to_target = match to {
        Currency::IDR => to_base * rates.rates.idr,
        Currency::USD => to_base * rates.rates.usd,
        Currency::EUR => to_base * rates.rates.eur,
        Currency::GBP => to_base * rates.rates.gbp,
        Currency::JPY => to_base * rates.rates.jpy,
        Currency::CHF => to_base * rates.rates.chf,
        Currency::SGD => to_base * rates.rates.sgd,
        Currency::CNY => to_base * rates.rates.cny,
        Currency::SAR => to_base * rates.rates.sar,
    };

    let result = Money::new_money(to, to_target);

    Ok(result)
}

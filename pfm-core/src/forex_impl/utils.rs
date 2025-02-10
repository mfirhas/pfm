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
        .map_err(|err| anyhow!("invalid currency code: {}", err))?;

    // 2. check if first part is in list of currency codes
    let code = Currency::from_str(currency_parts[0])
        .map_err(|err| anyhow!("Currency code invalid: {}", err))?;

    // 3. check if the code is in comma or dot separated for thousands.
    let mut is_comma_separated_code = COMMA_SEPARATED_CURRENCIES.contains(&code);

    // 4. validate format using regex
    let is_amount_validated = COMMA_SEPARATOR_REGEX.is_match(currency_parts[1]);
    if is_amount_validated {
        if !is_comma_separated_code {
            is_comma_separated_code = true;
        }
    } else {
        if !is_comma_separated_code {
            let is_dot_validated = DOT_SEPARATOR_REGEX.is_match(currency_parts[1]);
            if !is_dot_validated {
                return Err(anyhow!(ERROR_INVALID_AMOUNT_FORMAT));
            }
        } else {
            return Err(anyhow!(ERROR_INVALID_AMOUNT_FORMAT));
        }
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
    let decimal = Decimal::from_str(&amount)
        .map_err(|err| anyhow!("failed converting amount into decimal type: {}", err))?;

    match currency {
        Currency::IDR => Ok(Money::IDR(decimal)),
        Currency::USD => Ok(Money::USD(decimal)),
        Currency::EUR => Ok(Money::EUR(decimal)),
        Currency::GBP => Ok(Money::GBP(decimal)),
        Currency::JPY => Ok(Money::JPY(decimal)),
        Currency::CHF => Ok(Money::CHF(decimal)),
        Currency::SGD => Ok(Money::SGD(decimal)),
        Currency::CNY => Ok(Money::CNY(decimal)),
        Currency::SAR => Ok(Money::SAR(decimal)),
        _ => Err(anyhow!(
            "forex_impl: failed parsing Money from string, currency {} not supported.",
            currency.code()
        )),
    }
}

pub(crate) fn to_string(use_symbol: bool, money: Money) -> String {
    let currency_code: String = if use_symbol {
        money.symbol()
    } else {
        money.code()
    };

    let curr = match money {
        Money::IDR(_) => Currency::IDR,
        Money::USD(_) => Currency::USD,
        Money::EUR(_) => Currency::EUR,
        Money::GBP(_) => Currency::GBP,
        Money::JPY(_) => Currency::JPY,
        Money::CHF(_) => Currency::CHF,
        Money::SGD(_) => Currency::SGD,
        Money::CNY(_) => Currency::CNY,
        Money::SAR(_) => Currency::SAR,
    };

    let mut ac = if COMMA_SEPARATED_CURRENCIES.contains(&curr) {
        Accounting::new_from_seperator(currency_code.as_str(), 2, ",", ".")
    } else {
        Accounting::new_from_seperator(currency_code.as_str(), 2, ".", ",")
    };

    if use_symbol {
        ac.set_format("{s}{v}");
    } else {
        ac.set_format("{s} {v}");
    }

    let money_display = ac.format_money(money.amount());

    money_display
}

pub(crate) fn convert_currency(rates: &Rates, from: Money, to: Currencies) -> ForexResult<Money> {
    // 1. find the base currency, the one with value equals to 1.
    let base = rates.base;
    if from == base {
        return Ok(from);
    }

    // 2. divide from with its rate relative to base currency.
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

    // 3. multiply the above result with the rate of target conversion relative to base currency.
    let to_target = match to {
        Currencies::IDR => to_base * rates.rates.idr,
        Currencies::USD => to_base * rates.rates.usd,
        Currencies::EUR => to_base * rates.rates.eur,
        Currencies::GBP => to_base * rates.rates.gbp,
        Currencies::JPY => to_base * rates.rates.jpy,
        Currencies::CHF => to_base * rates.rates.chf,
        Currencies::SGD => to_base * rates.rates.sgd,
        Currencies::CNY => to_base * rates.rates.cny,
        Currencies::SAR => to_base * rates.rates.sar,
    };

    let result = Money::new_money(to, to_target);

    Ok(result)
}

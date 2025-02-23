use crate::forex::ForexError::InputError;
use crate::forex::*;
use accounting::Accounting;
use anyhow::anyhow;
use iso_currency::Currency as CurrencyLib;
use rust_decimal::Decimal;
use std::str::FromStr;

pub(crate) fn parse_str(input_money: &str) -> ForexResult<Money> {
    // 1. Check currency parts
    // currency parts <CODE> <major-unit.minor-unit>
    let currency_parts: Vec<&str> = input_money.split_whitespace().collect();
    if currency_parts.len() != 2 {
        return Err(InputError(anyhow!(ERROR_CURRENCY_PARTS)));
    }
    let currency = CurrencyLib::from_str(currency_parts[0])
        .map_err(|err| InputError(anyhow!("invalid currency code: {}", err)))?;

    // 2. check if first part is in list of currency codes
    let code = CurrencyLib::from_str(currency_parts[0])
        .map_err(|err| InputError(anyhow!("Currency code invalid: {}", err)))?;

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
                return Err(InputError(anyhow!(ERROR_INVALID_AMOUNT_FORMAT)));
            }
        } else {
            return Err(InputError(anyhow!(ERROR_INVALID_AMOUNT_FORMAT)));
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
    let decimal = Decimal::from_str(&amount).map_err(|err| {
        InputError(anyhow!(
            "failed converting amount into decimal type: {}",
            err
        ))
    })?;

    match currency {
        CurrencyLib::IDR => Ok(Money::IDR(decimal)),
        CurrencyLib::USD => Ok(Money::USD(decimal)),
        CurrencyLib::EUR => Ok(Money::EUR(decimal)),
        CurrencyLib::GBP => Ok(Money::GBP(decimal)),
        CurrencyLib::JPY => Ok(Money::JPY(decimal)),
        CurrencyLib::CHF => Ok(Money::CHF(decimal)),
        CurrencyLib::SGD => Ok(Money::SGD(decimal)),
        CurrencyLib::CNY => Ok(Money::CNY(decimal)),
        CurrencyLib::SAR => Ok(Money::SAR(decimal)),
        _ => Err(InputError(anyhow!(
            "forex_impl: failed parsing Money from string, currency {} not supported.",
            currency.code()
        ))),
    }
}

pub(crate) fn to_string(use_symbol: bool, money: Money) -> String {
    let currency_code: String = if use_symbol {
        money.symbol()
    } else {
        money.code()
    };

    let curr = match money {
        Money::IDR(_) => CurrencyLib::IDR,
        Money::USD(_) => CurrencyLib::USD,
        Money::EUR(_) => CurrencyLib::EUR,
        Money::GBP(_) => CurrencyLib::GBP,
        Money::JPY(_) => CurrencyLib::JPY,
        Money::CHF(_) => CurrencyLib::CHF,
        Money::SGD(_) => CurrencyLib::SGD,
        Money::CNY(_) => CurrencyLib::CNY,
        Money::SAR(_) => CurrencyLib::SAR,
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

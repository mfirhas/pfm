use std::{fmt::Display, str::FromStr};

use super::{
    currency::Currency,
    entity::Rates,
    interface::{AsClientError, ForexError, ForexResult},
};
use accounting::Accounting;
use anyhow::Context;
use iso_currency::Currency as CurrencyLib;
use lazy_static::lazy_static;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

use crate::global;

pub(crate) const ERROR_MONEY_FORMAT: &str = "The money must be written in ISO 4217 format: <CODE> <AMOUNT>. Amount may be separated by comma for thousands, and by dot for fraction.";

lazy_static! {
    /// Using ISO 4217 currency code with comma separated thousands(optional) and dot separated fraction.
    /// e.g.
    /// USD 1000
    /// USD 1,000
    /// USD 1,000.00
    /// IDR 5,000.235
    /// IDR 5,000,0223.445
    pub(crate) static ref MONEY_FORMAT_REGEX: regex::Regex =
        regex::Regex::new(r"^([A-Z]{3})\s+((?:\d{1,3}(?:,\d{3})*|\d+)(?:\.\d+)?)$").expect("failed compiling money format regex");
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Deserialize, EnumIter)]
pub enum Money {
    USD(Decimal),
    IDR(Decimal),
    EUR(Decimal),
    GBP(Decimal),
    JPY(Decimal),
    CHF(Decimal),
    SGD(Decimal),
    CNY(Decimal),
    SAR(Decimal),
}

impl Money {
    pub fn new(currency: &str, amount: &str) -> ForexResult<Self> {
        let curr = CurrencyLib::from_str(currency)
            .context("creating new Money with invalid currency")
            .as_client_err()?;
        let val = Decimal::from_str(amount)
            .context("Money convert str to Decimal")
            .as_client_err()?;

        match curr {
            CurrencyLib::IDR => Ok(Self::IDR(val)),
            CurrencyLib::USD => Ok(Self::USD(val)),
            CurrencyLib::EUR => Ok(Self::EUR(val)),
            CurrencyLib::GBP => Ok(Self::GBP(val)),
            CurrencyLib::JPY => Ok(Self::JPY(val)),
            CurrencyLib::CHF => Ok(Self::CHF(val)),
            CurrencyLib::SGD => Ok(Self::SGD(val)),
            CurrencyLib::CNY => Ok(Self::CNY(val)),
            CurrencyLib::SAR => Ok(Self::SAR(val)),
            _ => Err(ForexError::client_error("currency not supported yet")),
        }
    }

    pub fn new_money(currency: Currency, amount: Decimal) -> Money {
        match currency {
            Currency::IDR => Money::IDR(amount),
            Currency::USD => Money::USD(amount),
            Currency::EUR => Money::EUR(amount),
            Currency::GBP => Money::GBP(amount),
            Currency::JPY => Money::JPY(amount),
            Currency::CHF => Money::CHF(amount),
            Currency::SGD => Money::SGD(amount),
            Currency::CNY => Money::CNY(amount),
            Currency::SAR => Money::SAR(amount),
        }
    }

    pub fn currency(&self) -> Currency {
        match self {
            Self::IDR(_) => Currency::IDR,
            Self::USD(_) => Currency::USD,
            Self::EUR(_) => Currency::EUR,
            Self::GBP(_) => Currency::GBP,
            Self::JPY(_) => Currency::JPY,
            Self::CHF(_) => Currency::CHF,
            Self::SGD(_) => Currency::SGD,
            Self::CNY(_) => Currency::CNY,
            Self::SAR(_) => Currency::SAR,
        }
    }

    pub fn amount(&self) -> Decimal {
        match self {
            Self::IDR(val) => *val,
            Self::USD(val) => *val,
            Self::EUR(val) => *val,
            Self::GBP(val) => *val,
            Self::JPY(val) => *val,
            Self::CHF(val) => *val,
            Self::SGD(val) => *val,
            Self::CNY(val) => *val,
            Self::SAR(val) => *val,
        }
    }

    pub fn code(&self) -> String {
        match self {
            Self::IDR(_) => CurrencyLib::IDR.code().to_string(),
            Self::USD(_) => CurrencyLib::USD.code().to_string(),
            Self::EUR(_) => CurrencyLib::EUR.code().to_string(),
            Self::GBP(_) => CurrencyLib::GBP.code().to_string(),
            Self::JPY(_) => CurrencyLib::JPY.code().to_string(),
            Self::CHF(_) => CurrencyLib::CHF.code().to_string(),
            Self::SGD(_) => CurrencyLib::SGD.code().to_string(),
            Self::CNY(_) => CurrencyLib::CNY.code().to_string(),
            Self::SAR(_) => CurrencyLib::SAR.code().to_string(),
        }
    }

    pub fn symbol(&self) -> String {
        match self {
            Self::IDR(_) => CurrencyLib::IDR.symbol().to_string(),
            Self::USD(_) => CurrencyLib::USD.symbol().to_string(),
            Self::EUR(_) => CurrencyLib::EUR.symbol().to_string(),
            Self::GBP(_) => CurrencyLib::GBP.symbol().to_string(),
            Self::JPY(_) => CurrencyLib::JPY.symbol().to_string(),
            Self::CHF(_) => CurrencyLib::CHF.symbol().to_string(),
            Self::SGD(_) => CurrencyLib::SGD.symbol().to_string(),
            Self::CNY(_) => CurrencyLib::CNY.symbol().to_string(),
            Self::SAR(_) => CurrencyLib::SAR.symbol().to_string(),
        }
    }

    fn parse_str(input_money: &str) -> ForexResult<Money> {
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
            .context("Money parse_str to Decimal")
            .as_client_err()?;

        Ok(Money::new_money(currency, amount))
    }

    fn to_string(use_symbol: bool, money: Money) -> String {
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

    pub(super) fn convert(rates: &Rates, from: Money, to: Currency) -> ForexResult<Money> {
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
}

impl FromStr for Money {
    type Err = ForexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ret = Self::parse_str(s)?;
        Ok(ret)
    }
}

impl Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ret = Self::to_string(global::config().forex_use_symbol, *self);
        write!(f, "{}", ret)
    }
}

impl PartialEq<Currency> for Money {
    fn eq(&self, other: &Currency) -> bool {
        match (self, other) {
            (Money::IDR(_), Currency::IDR) => true,
            (Money::USD(_), Currency::USD) => true,
            (Money::EUR(_), Currency::EUR) => true,
            (Money::GBP(_), Currency::GBP) => true,
            (Money::JPY(_), Currency::JPY) => true,
            (Money::CHF(_), Currency::CHF) => true,
            (Money::SGD(_), Currency::SGD) => true,
            (Money::CNY(_), Currency::CNY) => true,
            (Money::SAR(_), Currency::SAR) => true,
            _ => false,
        }
    }
}

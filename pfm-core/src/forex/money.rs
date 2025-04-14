use std::{fmt::Display, str::FromStr};

use super::{
    currency::Currency,
    entity::RatesData,
    interface::{ForexError, ForexResult},
};
use crate::error::AsClientError;
use accounting::Accounting;
use anyhow::Context;
use iso_currency::Currency as CurrencyLib;
use lazy_static::lazy_static;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

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
    //// fiat

    // north america
    USD(Decimal),
    CAD(Decimal),

    // europe
    EUR(Decimal),
    GBP(Decimal),
    CHF(Decimal),
    RUB(Decimal),

    // east asia
    CNY(Decimal),
    JPY(Decimal),
    KRW(Decimal),
    HKD(Decimal),

    // south-east asia
    IDR(Decimal),
    MYR(Decimal),
    SGD(Decimal),
    THB(Decimal),

    // middle-east
    SAR(Decimal),
    AED(Decimal),
    KWD(Decimal),

    // south asia
    INR(Decimal),

    // apac
    AUD(Decimal),
    NZD(Decimal),

    //// precious metals
    XAU(Decimal), // troy ounce
    XAG(Decimal), // troy ounce
    XPT(Decimal), // troy ounce

    //// crypto
    BTC(Decimal),
    ETH(Decimal),
    SOL(Decimal),
    XRP(Decimal),
    ADA(Decimal),
}

impl Money {
    pub fn new(currency: &str, amount: &str) -> ForexResult<Self> {
        let quoted_curr = format!("\"{}\"", currency);
        let curr = serde_json::from_str(&quoted_curr)
            .with_context(|| {
                format!(
                    "creating new Money with invalid currency, currently supported currencies: {}",
                    Currency::to_comma_separated_list_str()
                )
            })
            .as_client_err()?;
        let val = Decimal::from_str(amount)
            .context("Money convert str to Decimal")
            .as_client_err()?;

        match curr {
            Currency::USD => Ok(Money::USD(val)),
            Currency::CAD => Ok(Money::CAD(val)),
            Currency::EUR => Ok(Money::EUR(val)),
            Currency::GBP => Ok(Money::GBP(val)),
            Currency::CHF => Ok(Money::CHF(val)),
            Currency::RUB => Ok(Money::RUB(val)),
            Currency::CNY => Ok(Money::CNY(val)),
            Currency::JPY => Ok(Money::JPY(val)),
            Currency::KRW => Ok(Money::KRW(val)),
            Currency::HKD => Ok(Money::HKD(val)),
            Currency::IDR => Ok(Money::IDR(val)),
            Currency::MYR => Ok(Money::MYR(val)),
            Currency::SGD => Ok(Money::SGD(val)),
            Currency::THB => Ok(Money::THB(val)),
            Currency::SAR => Ok(Money::SAR(val)),
            Currency::AED => Ok(Money::AED(val)),
            Currency::KWD => Ok(Money::KWD(val)),
            Currency::INR => Ok(Money::INR(val)),
            Currency::AUD => Ok(Money::AUD(val)),
            Currency::NZD => Ok(Money::NZD(val)),
            Currency::XAU => Ok(Money::XAU(val)),
            Currency::XAG => Ok(Money::XAG(val)),
            Currency::XPT => Ok(Money::XPT(val)),
            Currency::BTC => Ok(Money::BTC(val)),
            Currency::ETH => Ok(Money::ETH(val)),
            Currency::SOL => Ok(Money::SOL(val)),
            Currency::XRP => Ok(Money::XRP(val)),
            Currency::ADA => Ok(Money::ADA(val)),
        }
    }

    pub fn new_money(currency: Currency, amount: Decimal) -> Money {
        match currency {
            Currency::USD => Money::USD(amount),
            Currency::CAD => Money::CAD(amount),
            Currency::EUR => Money::EUR(amount),
            Currency::GBP => Money::GBP(amount),
            Currency::CHF => Money::CHF(amount),
            Currency::RUB => Money::RUB(amount),
            Currency::CNY => Money::CNY(amount),
            Currency::JPY => Money::JPY(amount),
            Currency::KRW => Money::KRW(amount),
            Currency::HKD => Money::HKD(amount),
            Currency::IDR => Money::IDR(amount),
            Currency::MYR => Money::MYR(amount),
            Currency::SGD => Money::SGD(amount),
            Currency::THB => Money::THB(amount),
            Currency::SAR => Money::SAR(amount),
            Currency::AED => Money::AED(amount),
            Currency::KWD => Money::KWD(amount),
            Currency::INR => Money::INR(amount),
            Currency::AUD => Money::AUD(amount),
            Currency::NZD => Money::NZD(amount),
            Currency::XAU => Money::XAU(amount),
            Currency::XAG => Money::XAG(amount),
            Currency::XPT => Money::XPT(amount),
            Currency::BTC => Money::BTC(amount),
            Currency::ETH => Money::ETH(amount),
            Currency::SOL => Money::SOL(amount),
            Currency::XRP => Money::XRP(amount),
            Currency::ADA => Money::ADA(amount),
        }
    }

    pub fn currency(&self) -> Currency {
        match self {
            Self::USD(_) => Currency::USD,
            Self::CAD(_) => Currency::CAD,
            Self::EUR(_) => Currency::EUR,
            Self::GBP(_) => Currency::GBP,
            Self::CHF(_) => Currency::CHF,
            Self::RUB(_) => Currency::RUB,
            Self::CNY(_) => Currency::CNY,
            Self::JPY(_) => Currency::JPY,
            Self::KRW(_) => Currency::KRW,
            Self::HKD(_) => Currency::HKD,
            Self::IDR(_) => Currency::IDR,
            Self::MYR(_) => Currency::MYR,
            Self::SGD(_) => Currency::SGD,
            Self::THB(_) => Currency::THB,
            Self::SAR(_) => Currency::SAR,
            Self::AED(_) => Currency::AED,
            Self::KWD(_) => Currency::KWD,
            Self::INR(_) => Currency::INR,
            Self::AUD(_) => Currency::AUD,
            Self::NZD(_) => Currency::NZD,
            Self::XAU(_) => Currency::XAU,
            Self::XAG(_) => Currency::XAG,
            Self::XPT(_) => Currency::XPT,
            Self::BTC(_) => Currency::BTC,
            Self::ETH(_) => Currency::ETH,
            Self::SOL(_) => Currency::SOL,
            Self::XRP(_) => Currency::XRP,
            Self::ADA(_) => Currency::ADA,
        }
    }

    pub fn amount(&self) -> Decimal {
        match self {
            Self::USD(val) => *val,
            Self::CAD(val) => *val,
            Self::EUR(val) => *val,
            Self::GBP(val) => *val,
            Self::CHF(val) => *val,
            Self::RUB(val) => *val,
            Self::CNY(val) => *val,
            Self::JPY(val) => *val,
            Self::KRW(val) => *val,
            Self::HKD(val) => *val,
            Self::IDR(val) => *val,
            Self::MYR(val) => *val,
            Self::SGD(val) => *val,
            Self::THB(val) => *val,
            Self::SAR(val) => *val,
            Self::AED(val) => *val,
            Self::KWD(val) => *val,
            Self::INR(val) => *val,
            Self::AUD(val) => *val,
            Self::NZD(val) => *val,
            Self::XAU(val) => *val,
            Self::XAG(val) => *val,
            Self::XPT(val) => *val,
            Self::BTC(val) => *val,
            Self::ETH(val) => *val,
            Self::SOL(val) => *val,
            Self::XRP(val) => *val,
            Self::ADA(val) => *val,
        }
    }

    pub fn code(&self) -> String {
        match self {
            Self::USD(_) => CurrencyLib::USD.code().to_string(),
            Self::CAD(_) => CurrencyLib::CAD.code().to_string(),
            Self::EUR(_) => CurrencyLib::EUR.code().to_string(),
            Self::GBP(_) => CurrencyLib::GBP.code().to_string(),
            Self::CHF(_) => CurrencyLib::CHF.code().to_string(),
            Self::RUB(_) => CurrencyLib::RUB.code().to_string(),
            Self::CNY(_) => CurrencyLib::CNY.code().to_string(),
            Self::JPY(_) => CurrencyLib::JPY.code().to_string(),
            Self::KRW(_) => CurrencyLib::KRW.code().to_string(),
            Self::HKD(_) => CurrencyLib::HKD.code().to_string(),
            Self::IDR(_) => CurrencyLib::IDR.code().to_string(),
            Self::MYR(_) => CurrencyLib::MYR.code().to_string(),
            Self::SGD(_) => CurrencyLib::SGD.code().to_string(),
            Self::THB(_) => CurrencyLib::THB.code().to_string(),
            Self::SAR(_) => CurrencyLib::SAR.code().to_string(),
            Self::AED(_) => CurrencyLib::AED.code().to_string(),
            Self::KWD(_) => CurrencyLib::KWD.code().to_string(),
            Self::INR(_) => CurrencyLib::INR.code().to_string(),
            Self::AUD(_) => CurrencyLib::AUD.code().to_string(),
            Self::NZD(_) => CurrencyLib::NZD.code().to_string(),
            Self::XAU(_) => CurrencyLib::XAU.code().to_string(),
            Self::XAG(_) => CurrencyLib::XAG.code().to_string(),
            Self::XPT(_) => CurrencyLib::XPT.code().to_string(),
            Self::BTC(_) => "BTC".to_string(),
            Self::ETH(_) => "ETH".to_string(),
            Self::SOL(_) => "SOL".to_string(),
            Self::XRP(_) => "XRP".to_string(),
            Self::ADA(_) => "ADA".to_string(),
        }
    }

    pub fn symbol(&self) -> String {
        match self {
            Self::USD(_) => CurrencyLib::USD.symbol().to_string(),
            Self::CAD(_) => CurrencyLib::CAD.symbol().to_string(),
            Self::EUR(_) => CurrencyLib::EUR.symbol().to_string(),
            Self::GBP(_) => CurrencyLib::GBP.symbol().to_string(),
            Self::CHF(_) => CurrencyLib::CHF.symbol().to_string(),
            Self::RUB(_) => CurrencyLib::RUB.symbol().to_string(),
            Self::CNY(_) => CurrencyLib::CNY.symbol().to_string(),
            Self::JPY(_) => CurrencyLib::JPY.symbol().to_string(),
            Self::KRW(_) => CurrencyLib::KRW.symbol().to_string(),
            Self::HKD(_) => CurrencyLib::HKD.symbol().to_string(),
            Self::IDR(_) => CurrencyLib::IDR.symbol().to_string(),
            Self::MYR(_) => CurrencyLib::MYR.symbol().to_string(),
            Self::SGD(_) => CurrencyLib::SGD.symbol().to_string(),
            Self::THB(_) => CurrencyLib::THB.symbol().to_string(),
            Self::SAR(_) => CurrencyLib::SAR.symbol().to_string(),
            Self::AED(_) => CurrencyLib::AED.symbol().to_string(),
            Self::KWD(_) => CurrencyLib::KWD.symbol().to_string(),
            Self::INR(_) => CurrencyLib::INR.symbol().to_string(),
            Self::AUD(_) => CurrencyLib::AUD.symbol().to_string(),
            Self::NZD(_) => CurrencyLib::NZD.symbol().to_string(),
            Self::XAU(_) => CurrencyLib::XAU.symbol().to_string(),
            Self::XAG(_) => CurrencyLib::XAG.symbol().to_string(),
            Self::XPT(_) => CurrencyLib::XPT.symbol().to_string(),
            Self::BTC(_) => "₿".to_string(),
            Self::ETH(_) => "Ξ".to_string(),
            Self::SOL(_) => "◎".to_string(),
            Self::XRP(_) => "✕".to_string(),
            Self::ADA(_) => "₳".to_string(),
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

    fn to_string(&self, use_symbol: bool) -> String {
        let currency_code: String = if use_symbol {
            self.symbol()
        } else {
            self.code()
        };

        let mut ac = Accounting::new_from_seperator(currency_code.as_str(), 2, ",", ".");

        if use_symbol {
            ac.set_format("{s}{v}");
        } else {
            ac.set_format("{s} {v}");
        }

        let money_display = ac.format_money(self.amount());

        money_display
    }

    pub(super) fn convert(rates: &RatesData, from: Money, to: Currency) -> ForexResult<Money> {
        if from.currency() == to {
            return Ok(from);
        }

        // 1. divide from with its rate relative to base currency.
        let to_base = match from {
            Money::USD(amount) => amount.checked_div(rates.usd).unwrap_or_default(),
            Money::CAD(amount) => amount.checked_div(rates.cad).unwrap_or_default(),
            Money::EUR(amount) => amount.checked_div(rates.eur).unwrap_or_default(),
            Money::GBP(amount) => amount.checked_div(rates.gbp).unwrap_or_default(),
            Money::CHF(amount) => amount.checked_div(rates.chf).unwrap_or_default(),
            Money::RUB(amount) => amount.checked_div(rates.rub).unwrap_or_default(),
            Money::CNY(amount) => amount.checked_div(rates.cny).unwrap_or_default(),
            Money::JPY(amount) => amount.checked_div(rates.jpy).unwrap_or_default(),
            Money::KRW(amount) => amount.checked_div(rates.krw).unwrap_or_default(),
            Money::HKD(amount) => amount.checked_div(rates.hkd).unwrap_or_default(),
            Money::IDR(amount) => amount.checked_div(rates.idr).unwrap_or_default(),
            Money::MYR(amount) => amount.checked_div(rates.myr).unwrap_or_default(),
            Money::SGD(amount) => amount.checked_div(rates.sgd).unwrap_or_default(),
            Money::THB(amount) => amount.checked_div(rates.thb).unwrap_or_default(),
            Money::SAR(amount) => amount.checked_div(rates.sar).unwrap_or_default(),
            Money::AED(amount) => amount.checked_div(rates.aed).unwrap_or_default(),
            Money::KWD(amount) => amount.checked_div(rates.kwd).unwrap_or_default(),
            Money::INR(amount) => amount.checked_div(rates.inr).unwrap_or_default(),
            Money::AUD(amount) => amount.checked_div(rates.aud).unwrap_or_default(),
            Money::NZD(amount) => amount.checked_div(rates.nzd).unwrap_or_default(),
            Money::XAU(amount) => amount.checked_div(rates.xau).unwrap_or_default(),
            Money::XAG(amount) => amount.checked_div(rates.xag).unwrap_or_default(),
            Money::XPT(amount) => amount.checked_div(rates.xpt).unwrap_or_default(),
            Money::BTC(amount) => amount.checked_div(rates.btc).unwrap_or_default(),
            Money::ETH(amount) => amount.checked_div(rates.eth).unwrap_or_default(),
            Money::SOL(amount) => amount.checked_div(rates.sol).unwrap_or_default(),
            Money::XRP(amount) => amount.checked_div(rates.xrp).unwrap_or_default(),
            Money::ADA(amount) => amount.checked_div(rates.ada).unwrap_or_default(),
        };

        // 2. multiply the above result with the rate of target conversion relative to base currency.
        let to_target = match to {
            Currency::USD => to_base * rates.usd,
            Currency::CAD => to_base * rates.cad,
            Currency::EUR => to_base * rates.eur,
            Currency::GBP => to_base * rates.gbp,
            Currency::CHF => to_base * rates.chf,
            Currency::RUB => to_base * rates.rub,
            Currency::CNY => to_base * rates.cny,
            Currency::JPY => to_base * rates.jpy,
            Currency::KRW => to_base * rates.krw,
            Currency::HKD => to_base * rates.hkd,
            Currency::IDR => to_base * rates.idr,
            Currency::MYR => to_base * rates.myr,
            Currency::SGD => to_base * rates.sgd,
            Currency::THB => to_base * rates.thb,
            Currency::SAR => to_base * rates.sar,
            Currency::AED => to_base * rates.aed,
            Currency::KWD => to_base * rates.kwd,
            Currency::INR => to_base * rates.inr,
            Currency::AUD => to_base * rates.aud,
            Currency::NZD => to_base * rates.nzd,
            Currency::XAU => to_base * rates.xau,
            Currency::XAG => to_base * rates.xag,
            Currency::XPT => to_base * rates.xpt,
            Currency::BTC => to_base * rates.btc,
            Currency::ETH => to_base * rates.eth,
            Currency::SOL => to_base * rates.sol,
            Currency::XRP => to_base * rates.xrp,
            Currency::ADA => to_base * rates.ada,
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
        let ret = self.to_string(global::config().forex_use_symbol);
        write!(f, "{}", ret)
    }
}

impl From<Currency> for Money {
    fn from(value: Currency) -> Self {
        match value {
            Currency::USD => Money::USD(dec!(0)),
            Currency::CAD => Money::CAD(dec!(0)),
            Currency::EUR => Money::EUR(dec!(0)),
            Currency::GBP => Money::GBP(dec!(0)),
            Currency::CHF => Money::CHF(dec!(0)),
            Currency::RUB => Money::RUB(dec!(0)),
            Currency::CNY => Money::CNY(dec!(0)),
            Currency::JPY => Money::JPY(dec!(0)),
            Currency::KRW => Money::KRW(dec!(0)),
            Currency::HKD => Money::HKD(dec!(0)),
            Currency::IDR => Money::IDR(dec!(0)),
            Currency::MYR => Money::MYR(dec!(0)),
            Currency::SGD => Money::SGD(dec!(0)),
            Currency::THB => Money::THB(dec!(0)),
            Currency::SAR => Money::SAR(dec!(0)),
            Currency::AED => Money::AED(dec!(0)),
            Currency::KWD => Money::KWD(dec!(0)),
            Currency::INR => Money::INR(dec!(0)),
            Currency::AUD => Money::AUD(dec!(0)),
            Currency::NZD => Money::NZD(dec!(0)),
            Currency::XAU => Money::XAU(dec!(0)),
            Currency::XAG => Money::XAG(dec!(0)),
            Currency::XPT => Money::XPT(dec!(0)),
            Currency::BTC => Money::BTC(dec!(0)),
            Currency::ETH => Money::ETH(dec!(0)),
            Currency::SOL => Money::SOL(dec!(0)),
            Currency::XRP => Money::XRP(dec!(0)),
            Currency::ADA => Money::ADA(dec!(0)),
        }
    }
}

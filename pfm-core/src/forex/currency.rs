use anyhow::Context;
use std::{fmt::Display, str::FromStr};

use iso_currency::Currency as CurrencyLib;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

use super::{interface::ForexError, money::Money};
use crate::error::AsClientError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter)]
pub enum Currency {
    //// fiat

    // north america
    USD,
    CAD,

    // europe
    EUR,
    GBP,
    CHF,
    RUB,

    // east asia
    CNY,
    JPY,
    KRW,
    HKD,

    // south-east asia
    IDR,
    MYR,
    SGD,
    THB,

    // middle-east
    SAR,
    AED,
    KWD,

    // south asia
    INR,

    // apac
    AUD,
    NZD,

    //// precious metals
    XAU, // troy ounce
    XAG, // troy ounce
    XPT, // troy ounce

    //// crypto
    BTC,
    ETH,
    SOL,
    XRP,
    ADA,
}

impl Currency {
    pub fn code(&self) -> &'static str {
        match self {
            Self::USD => CurrencyLib::USD.code(),
            Self::CAD => CurrencyLib::CAD.code(),
            Self::EUR => CurrencyLib::EUR.code(),
            Self::GBP => CurrencyLib::GBP.code(),
            Self::CHF => CurrencyLib::CHF.code(),
            Self::RUB => CurrencyLib::RUB.code(),
            Self::CNY => CurrencyLib::CNY.code(),
            Self::JPY => CurrencyLib::JPY.code(),
            Self::KRW => CurrencyLib::KRW.code(),
            Self::HKD => CurrencyLib::HKD.code(),
            Self::IDR => CurrencyLib::IDR.code(),
            Self::MYR => CurrencyLib::MYR.code(),
            Self::SGD => CurrencyLib::SGD.code(),
            Self::THB => CurrencyLib::THB.code(),
            Self::SAR => CurrencyLib::SAR.code(),
            Self::AED => CurrencyLib::AED.code(),
            Self::KWD => CurrencyLib::KWD.code(),
            Self::INR => CurrencyLib::INR.code(),
            Self::AUD => CurrencyLib::AUD.code(),
            Self::NZD => CurrencyLib::NZD.code(),
            Self::XAU => CurrencyLib::XAU.code(),
            Self::XAG => CurrencyLib::XAG.code(),
            Self::XPT => CurrencyLib::XPT.code(),
            Self::BTC => "BTC",
            Self::ETH => "ETH",
            Self::SOL => "SOL",
            Self::XRP => "XRP",
            Self::ADA => "ADA",
        }
    }

    pub fn to_comma_separated_list_str() -> String {
        let ret = Currency::iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(",");
        ret
    }

    pub fn to_comma_separated_pair_list_str(base: Currency) -> String {
        Currency::iter()
            .filter(|&c| c != base)
            .map(|c| format!("{}{}", base.code(), format!("{:?}", c)))
            .collect::<Vec<_>>()
            .join(",")
    }

    pub fn currencies_count() -> usize {
        Currency::iter().count() as usize
    }
}

impl FromStr for Currency {
    type Err = ForexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let quoted_curr = format!("\"{}\"", s);
        let curr = serde_json::from_str(&quoted_curr)
            .with_context(|| {
                format!(
                    "currency parsing from str invalid, currently supported currencies: {}",
                    Currency::to_comma_separated_list_str()
                )
            })
            .as_client_err()?;

        Ok(curr)
    }
}

impl Default for Currency {
    fn default() -> Self {
        Self::USD
    }
}

impl From<Money> for Currency {
    fn from(value: Money) -> Self {
        match value {
            Money::USD(_) => Self::USD,
            Money::CAD(_) => Self::CAD,
            Money::EUR(_) => Self::EUR,
            Money::GBP(_) => Self::GBP,
            Money::CHF(_) => Self::CHF,
            Money::RUB(_) => Self::RUB,
            Money::CNY(_) => Self::CNY,
            Money::JPY(_) => Self::JPY,
            Money::KRW(_) => Self::KRW,
            Money::HKD(_) => Self::HKD,
            Money::IDR(_) => Self::IDR,
            Money::MYR(_) => Self::MYR,
            Money::SGD(_) => Self::SGD,
            Money::THB(_) => Self::THB,
            Money::SAR(_) => Self::SAR,
            Money::AED(_) => Self::AED,
            Money::KWD(_) => Self::KWD,
            Money::INR(_) => Self::INR,
            Money::AUD(_) => Self::AUD,
            Money::NZD(_) => Self::NZD,
            Money::XAU(_) => Self::XAU,
            Money::XAG(_) => Self::XAG,
            Money::XPT(_) => Self::XPT,
            Money::BTC(_) => Self::BTC,
            Money::ETH(_) => Self::ETH,
            Money::SOL(_) => Self::SOL,
            Money::XRP(_) => Self::XRP,
            Money::ADA(_) => Self::ADA,
        }
    }
}

impl Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let r = match self {
            Self::USD => CurrencyLib::USD.code(),
            Self::CAD => CurrencyLib::CAD.code(),
            Self::EUR => CurrencyLib::EUR.code(),
            Self::GBP => CurrencyLib::GBP.code(),
            Self::CHF => CurrencyLib::CHF.code(),
            Self::RUB => CurrencyLib::RUB.code(),
            Self::CNY => CurrencyLib::CNY.code(),
            Self::JPY => CurrencyLib::JPY.code(),
            Self::KRW => CurrencyLib::KRW.code(),
            Self::HKD => CurrencyLib::HKD.code(),
            Self::IDR => CurrencyLib::IDR.code(),
            Self::MYR => CurrencyLib::MYR.code(),
            Self::SGD => CurrencyLib::SGD.code(),
            Self::THB => CurrencyLib::THB.code(),
            Self::SAR => CurrencyLib::SAR.code(),
            Self::AED => CurrencyLib::AED.code(),
            Self::KWD => CurrencyLib::KWD.code(),
            Self::INR => CurrencyLib::INR.code(),
            Self::AUD => CurrencyLib::AUD.code(),
            Self::NZD => CurrencyLib::NZD.code(),
            Self::XAU => CurrencyLib::XAU.code(),
            Self::XAG => CurrencyLib::XAG.code(),
            Self::XPT => CurrencyLib::XPT.code(),
            Self::BTC => "BTC",
            Self::ETH => "ETH",
            Self::SOL => "SOL",
            Self::XRP => "XRP",
            Self::ADA => "ADA",
        };

        write!(f, "{}", r)
    }
}

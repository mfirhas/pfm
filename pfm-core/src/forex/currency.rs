use anyhow::Context;
use std::{fmt::Display, str::FromStr};

use iso_currency::Currency as CurrencyLib;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

use super::{
    interface::{AsClientError, ForexError},
    money::Money,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIter)]
pub enum Currency {
    USD,
    IDR,
    EUR,
    GBP,
    JPY,
    CHF,
    SGD,
    CNY,
    SAR,
}

impl Currency {
    pub fn code(&self) -> &'static str {
        match self {
            Self::IDR => CurrencyLib::IDR.code(),
            Self::USD => CurrencyLib::USD.code(),
            Self::EUR => CurrencyLib::EUR.code(),
            Self::GBP => CurrencyLib::GBP.code(),
            Self::JPY => CurrencyLib::JPY.code(),
            Self::CHF => CurrencyLib::CHF.code(),
            Self::SGD => CurrencyLib::SGD.code(),
            Self::CNY => CurrencyLib::CNY.code(),
            Self::SAR => CurrencyLib::SAR.code(),
        }
    }

    pub fn to_comma_separated_list_str(&self) -> String {
        let ret = Currency::iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(",");
        ret
    }
}

impl FromStr for Currency {
    type Err = ForexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let curr: CurrencyLib = s
            .parse()
            .context("currency parsing from str")
            .as_client_err()?;

        match curr {
            CurrencyLib::IDR => Ok(Self::IDR),
            CurrencyLib::USD => Ok(Self::USD),
            CurrencyLib::EUR => Ok(Self::EUR),
            CurrencyLib::GBP => Ok(Self::GBP),
            CurrencyLib::JPY => Ok(Self::JPY),
            CurrencyLib::CHF => Ok(Self::CHF),
            CurrencyLib::SGD => Ok(Self::SGD),
            CurrencyLib::CNY => Ok(Self::CNY),
            CurrencyLib::SAR => Ok(Self::SAR),
            _ => Err(ForexError::client_error("currency not supported")),
        }
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
            Money::IDR(_) => Self::IDR,
            Money::USD(_) => Self::USD,
            Money::EUR(_) => Self::EUR,
            Money::GBP(_) => Self::GBP,
            Money::JPY(_) => Self::JPY,
            Money::CHF(_) => Self::CHF,
            Money::SGD(_) => Self::SGD,
            Money::CNY(_) => Self::CNY,
            Money::SAR(_) => Self::SAR,
        }
    }
}

impl Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let r = match self {
            Self::IDR => CurrencyLib::IDR.code(),
            Self::USD => CurrencyLib::USD.code(),
            Self::EUR => CurrencyLib::EUR.code(),
            Self::GBP => CurrencyLib::GBP.code(),
            Self::JPY => CurrencyLib::JPY.code(),
            Self::CHF => CurrencyLib::CHF.code(),
            Self::SGD => CurrencyLib::SGD.code(),
            Self::CNY => CurrencyLib::CNY.code(),
            Self::SAR => CurrencyLib::SAR.code(),
        };

        write!(f, "{}", r)
    }
}

/// https://github.com/fawazahmed0/exchange-api
pub mod exchange_api;
#[cfg(test)]
mod exchange_api_test;

/// https://currencyapi.com
pub mod currency_api;
#[cfg(test)]
mod currency_api_test;

/// https://openexchangerates.org/
pub mod open_exchange_api;
#[cfg(test)]
mod open_exchange_api_test;

pub(crate) mod utils;

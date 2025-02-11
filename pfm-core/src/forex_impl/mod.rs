/// https://github.com/fawazahmed0/exchange-api
pub(crate) mod exchange_api;
#[cfg(test)]
mod exchange_api_test;

/// https://currencyapi.com
pub(crate) mod currency_api;
// #[cfg(test)]
// mod currency_api_test;

/// https://openexchangerates.org/
pub(crate) mod open_exchange_api;
#[cfg(test)]
mod open_exchange_api_test;

pub(crate) mod utils;

use serde::Deserialize;
use std::{fmt::Debug, sync::LazyLock};

use crate::utils;

/// Get instantiated global config object.
pub fn config() -> &'static Config {
    &CONFIG
}

static CONFIG: LazyLock<Config> =
    LazyLock::new(|| init_config().expect("global config: failed initializing config"));

const ENV_PREFIX: &str = "CORE_";

fn init_config<CFG>() -> Result<CFG, anyhow::Error>
where
    CFG: for<'de> Deserialize<'de> + Debug + Clone,
{
    let cfg = utils::get_config(ENV_PREFIX);

    cfg
}

/// Configurations for pfm-core
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Using symbol or code for displaying money.
    #[serde(alias = "CORE_FOREX_USE_SYMBOL", default)]
    pub forex_use_symbol: bool,

    /// API key for https://currencyapi.com
    #[serde(alias = "CORE_FOREX_CURRENCY_API_KEY")]
    pub forex_currency_api_key: String,

    /// API key for https://openexchangerates.org
    #[serde(alias = "CORE_FOREX_OPEN_EXCHANGE_API_KEY")]
    pub forex_open_exchange_api_key: String,

    #[serde(alias = "CORE_FOREX_CURRENCYBEACON_API_KEY")]
    pub forex_currencybeacon_api_key: String,

    #[serde(alias = "CORE_FOREX_TRADERMADE_API_KEY")]
    pub forex_tradermade_api_key: String,

    #[serde(alias = "CORE_FOREX_TWELVEDATA_API_KEY")]
    pub forex_twelvedata_api_key: String,
}

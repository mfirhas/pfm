// global.rs contains global variables

use anyhow::Result;
use configrs::config::Config as config_rs;
use lazy_static::lazy_static;
use reqwest::Client;
use serde::Deserialize;
use std::{fmt::Debug, time::Duration};

const ENV_PREFIX: &str = "CORE_";
const ERROR_PREFIX: &str = "[GLOBAL]";

/// path to .env file for development
pub(super) const DEV_ENV_PATH: &str = "./src/core.env";

/// Get instantiated global http client object.
pub(crate) fn http_client() -> &'static Client {
    &HTTP_CLIENT
}

/// Get instantiated global config object.
pub(crate) fn config() -> &'static Config {
    &CONFIG
}

lazy_static! {
    static ref CONFIG: Config = init_config().expect("failed init core config");
    static ref HTTP_CLIENT: Client = init_http_client().expect("failed init core http client");
}

fn init_http_client() -> Result<reqwest::Client, anyhow::Error> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .pool_idle_timeout(Duration::from_secs(300))
        .pool_max_idle_per_host(32)
        .build()
        .map_err(|err| anyhow::anyhow!("{} failed creating http client: {}", ERROR_PREFIX, err));

    client
}

fn init_config<CFG>() -> Result<CFG, anyhow::Error>
where
    CFG: for<'de> Deserialize<'de> + Debug + Clone,
{
    let cfg = config_rs::new().with_env_prefix(ENV_PREFIX);

    // for local development config from file
    if cfg!(debug_assertions) {
        let cfg = cfg.with_env(DEV_ENV_PATH).build::<CFG>().map_err(|err| {
            anyhow::anyhow!(
                "{} failed parsing dev config from file {}: {}",
                ERROR_PREFIX,
                ENV_PREFIX,
                err
            )
        });

        return cfg;
    }

    let cfg = cfg
        .build::<CFG>()
        .map_err(|err| anyhow::anyhow!("{} failed parsing env config: {}", ERROR_PREFIX, err));

    cfg
}

/// Configurations
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Config {
    /// Using symbol or code for displaying money.
    #[serde(alias = "CORE_FOREX_USE_SYMBOL", default)]
    pub forex_use_symbol: bool,

    /// API key for https://currencyapi.com
    #[serde(alias = "CORE_FOREX_CURRENCY_API_KEY")]
    pub forex_currency_api_key: String,

    /// API key for https://openexchangerates.org
    #[serde(alias = "CORE_FOREX_OPEN_EXCHANGE_API_KEY")]
    pub forex_open_exchange_api_key: String,
}

#[cfg(test)]
mod global_tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_dev_dir() {
        let path = Path::new(DEV_ENV_PATH);

        assert!(path.exists());
    }

    #[test]
    fn test_config() {
        let cfg = init_config::<Config>();
        dbg!(&cfg);
        assert!(cfg.is_ok());
    }
}

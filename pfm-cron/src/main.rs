use std::{
    future::Future,
    pin::{self, Pin},
};

use anyhow::anyhow;
use chrono::{DateTime, Utc};
use configrs::config::Config as configrs;
use pfm_core::{
    forex::{self, Currencies, ForexHistoricalRates, ForexRates, ForexStorage},
    forex_impl, forex_storage_impl, global,
};
use serde::Deserialize;
use tokio::signal;
use tokio_cron_scheduler::{Job, JobScheduler};

const ERROR_PREFIX: &str = "[CRON]";
const DEV_LOCAL_ENV_PATH: &str = "./pfm-cron/cron.env";

async fn poll_latest_rates(fx: impl ForexRates, fs: impl ForexStorage, base: Currencies) {
    let _ = forex::poll_rates(&fx, &fs, base).await;
}

async fn poll_historical_rates(
    fx: impl ForexHistoricalRates,
    fs: impl ForexStorage,
    date: DateTime<Utc>,
    base: Currencies,
) {
    let _ = forex::poll_historical_rates(&fx, &fs, date, base).await;
}

fn dep_forex_impl() -> forex_impl::open_exchange_api::Api {
    let core_cfg = global::config();
    let http_client = global::http_client();

    let forex_impl =
        forex_impl::open_exchange_api::Api::new(&core_cfg.forex_open_exchange_api_key, http_client);

    forex_impl
}

fn dep_storage_impl() -> forex_storage_impl::forex_storage::ForexStorageImpl {
    let storage = global::storage_fs();

    let storage_impl = forex_storage_impl::forex_storage::ForexStorageImpl::new(storage);

    storage_impl
}

#[tokio::main]
async fn main() {
    let cfg = init_config().expect("failed initializing config");

    let scheduler = JobScheduler::new()
        .await
        .expect("failed initializing JobScheduler");

    let poll_rates_job = Job::new_async(&cfg.cron_tab_poll_rates, |_uuid, _lock| {
        let forex = dep_forex_impl();
        let storage = dep_storage_impl();
        Box::pin(poll_latest_rates(forex, storage, Currencies::USD))
    })
    .expect("failed initializing poll_rates_job");

    let poll_historical_rates_job =
        Job::new_async(&cfg.cron_tab_poll_historical_rates, |_uuid, _lock| {
            let forex = dep_forex_impl();
            let storage = dep_storage_impl();

            Box::pin(poll_historical_rates(
                forex,
                storage,
                Utc::now(),
                Currencies::USD,
            ))
        })
        .expect("failed initializing poll_historical_rates_job");

    scheduler
        .add(poll_rates_job)
        .await
        .expect("failed adding job1");

    scheduler
        .add(poll_historical_rates_job)
        .await
        .expect("failed adding job2");

    scheduler.start().await.expect("failed starting scheduler");

    signal::ctrl_c()
        .await
        .expect("failed reading interrupting signal");

    println!("[CRON] Shutting down gracefully...");
}

fn init_config() -> Result<Config, anyhow::Error> {
    let cfg = configrs::new().with_env_prefix("CRON_");
    if cfg!(debug_assertions) {
        let ret = cfg
            .with_env(DEV_LOCAL_ENV_PATH)
            .build::<Config>()
            .map_err(|err| {
                anyhow!(
                    "{} failed reading local config at {}: {}",
                    ERROR_PREFIX,
                    DEV_LOCAL_ENV_PATH,
                    err
                )
            });
        return ret;
    }

    let ret = cfg
        .build::<Config>()
        .map_err(|err| anyhow!("{} failed parsing env: {}", ERROR_PREFIX, err));

    ret
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Config {
    /// cron tab for poll latest rates
    #[serde(alias = "CRON_TAB_POLL_RATES")]
    pub cron_tab_poll_rates: String,

    /// cron tab for poll historical rates
    #[serde(alias = "CRON_TAB_POLL_HISTORICAL_RATES")]
    pub cron_tab_poll_historical_rates: String,
}

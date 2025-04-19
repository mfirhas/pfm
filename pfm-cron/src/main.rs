use anyhow::Result;
use pfm_core::{forex_impl, global};
use pfm_utils::tracing_util;
use serde::Deserialize;
use tokio::signal;
use tokio_cron_scheduler::JobScheduler;

mod job;

const ENV_PREFIX: &str = "CRON_";

#[tokio::main]
async fn main() {
    tracing_util::init_tracing("pfm-cron");

    let core_cfg = global::config();
    let cron_config = init_config().expect("cron initializing config");

    // dependencies
    let forex_api = forex_impl::currencybeacon::Api::new(
        &core_cfg.forex_currencybeacon_api_key,
        global::http_client(),
    );
    let forex_storage = forex_impl::forex_storage::ForexStorageImpl::new(global::storage_fs());
    // END

    let scheduler = JobScheduler::new()
        .await
        .expect("failed initializing JobScheduler");

    // registering jobs
    let scheduler = job::poll_latest_rates_job(
        &scheduler,
        &cron_config,
        forex_api.clone(),
        forex_storage.clone(),
    )
    .await
    .expect("cron registering poll_latest_rates_job");

    let scheduler = job::poll_historical_rates_job(
        &scheduler,
        &cron_config,
        forex_api,
        forex_storage.clone(),
        forex_storage,
    )
    .await
    .expect("cron registering poll_historical_rates_job");
    // END

    scheduler.start().await.expect("failed starting scheduler");

    signal::ctrl_c()
        .await
        .expect("failed reading interrupting signal");

    tracing::info!("cron Shutting down gracefully...");
}

fn init_config() -> Result<Config, anyhow::Error> {
    let cfg = pfm_core::utils::get_config::<Config>(ENV_PREFIX);

    cfg
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Config {
    #[serde(alias = "CRON_TAB_POLL_RATES")]
    pub crontab_poll_rates: String,

    #[serde(alias = "CRON_ENABLE_POLL_RATES")]
    pub cron_enable_poll_rates: bool,

    #[serde(alias = "CRON_TAB_POLL_HISTORICAL_RATES")]
    pub crontab_poll_historical_rates: String,

    #[serde(alias = "CRON_ENABLE_POLL_HISTORICAL_RATES")]
    pub cron_enable_poll_historical_rates: bool,
}

use anyhow::{anyhow, Result};
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
const DEV_ENV_FILE_PATH: &str = ".env";

#[tokio::main]
async fn main() {
    let cfg = init_config().expect("failed initializing config");
    let core_cfg = global::config();

    let scheduler = JobScheduler::new()
        .await
        .expect("failed initializing JobScheduler");

    let poll_latest_rates_sge_job = poll_latest_rates_sge_job(&cfg, core_cfg)
        .expect("failed initializing poll_latest_rates_sge_job");

    let poll_latest_rates_lbma_am_job = poll_latest_rates_lbma_am_job(&cfg, core_cfg)
        .expect("failed initializing poll_latest_rates_lbma_am_job");

    let poll_latest_rates_lbma_pm_job = poll_latest_rates_lbma_pm_job(&cfg, core_cfg)
        .expect("failed initializing poll_latest_rates_lbma_pm_job");

    let poll_historical_rates_job = poll_historical_rates_job(&cfg, core_cfg)
        .expect("failed initializing poll_historical_rates_job");

    let scheduler = register_cron_jobs(
        &scheduler,
        &cfg,
        poll_latest_rates_sge_job,
        poll_latest_rates_lbma_am_job,
        poll_latest_rates_lbma_pm_job,
        poll_historical_rates_job,
    )
    .await
    .expect("failed registering jobs");

    scheduler.start().await.expect("failed starting scheduler");

    signal::ctrl_c()
        .await
        .expect("failed reading interrupting signal");

    println!("{} Shutting down gracefully...", ERROR_PREFIX);
}

fn init_config() -> Result<Config, anyhow::Error> {
    let cfg = configrs::new().with_env_prefix("CRON_");
    if cfg!(debug_assertions) {
        let workspace_dir = pfm_core::utils::find_workspace_root()?;
        let dev_config_file = workspace_dir.join(DEV_ENV_FILE_PATH);
        let ret = cfg
            .with_env(&dev_config_file)
            .build::<Config>()
            .map_err(|err| {
                anyhow!(
                    "{} failed reading local config at {:?}: {}",
                    ERROR_PREFIX,
                    dev_config_file.as_path(),
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
    // #[serde(alias = "CRON_TAB_POLL_RATES")]
    // pub cron_tab_poll_rates: String,

    #[serde(alias = "CRON_TAB_POLL_RATES_SGE")]
    pub cron_tab_poll_rates_sge: String,

    #[serde(alias = "CRON_TAB_POLL_RATES_LBMA_AM")]
    pub cron_tab_poll_rates_lbma_am: String,

    #[serde(alias = "CRON_TAB_POLL_RATES_LBMA_PM")]
    pub cron_tab_poll_rates_lbma_pm: String,

    #[serde(alias = "CRON_ENABLE_POLL_RATES")]
    pub cron_enable_poll_rates: bool,

    /// cron tab for poll historical rates
    #[serde(alias = "CRON_TAB_POLL_HISTORICAL_RATES")]
    pub cron_tab_poll_historical_rates: String,

    #[serde(alias = "CRON_ENABLE_POLL_HISTORICAL_RATES")]
    pub cron_enable_poll_historical_rates: bool,
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

/// register and remove cron jobs here
async fn register_cron_jobs<'a>(
    scheduler: &'a JobScheduler,
    cron_cfg: &'a Config,
    poll_latest_rates_sge_job: Job,
    poll_latest_rates_lbma_am: Job,
    poll_latest_rates_lbma_pm: Job,
    poll_historical_rates_job: Job,
) -> Result<&'a JobScheduler, anyhow::Error> {
    let poll_rates_sge_job_id = &poll_latest_rates_sge_job.guid();
    scheduler
        .add(poll_latest_rates_sge_job)
        .await
        .map_err(|err| {
            anyhow!(
                "{} failed adding poll_latest_rates_sge_job: {}",
                ERROR_PREFIX,
                err
            )
        })?;

    let poll_rates_lbma_am_job_id = &poll_latest_rates_lbma_am.guid();
    scheduler
        .add(poll_latest_rates_lbma_am)
        .await
        .map_err(|err| {
            anyhow!(
                "{} failed adding poll_latest_rates_lbma_am: {}",
                ERROR_PREFIX,
                err
            )
        })?;

    let poll_rates_lbma_pm_job_id = &poll_latest_rates_lbma_pm.guid();
    scheduler
        .add(poll_latest_rates_lbma_pm)
        .await
        .map_err(|err| {
            anyhow!(
                "{} failed adding poll_latest_rates_lbma_pm: {}",
                ERROR_PREFIX,
                err
            )
        })?;

    if !cron_cfg.cron_enable_poll_rates {
        scheduler
            .remove(poll_rates_sge_job_id)
            .await
            .map_err(|err| {
                anyhow!(
                    "{} failed removing poll_rates_sge_job_id :{}",
                    ERROR_PREFIX,
                    err
                )
            })?;

        scheduler
            .remove(poll_rates_lbma_am_job_id)
            .await
            .map_err(|err| {
                anyhow!(
                    "{} failed removing poll_rates_lbma_am_job_id :{}",
                    ERROR_PREFIX,
                    err
                )
            })?;

        scheduler
            .remove(poll_rates_lbma_pm_job_id)
            .await
            .map_err(|err| {
                anyhow!(
                    "{} failed removing poll_rates_lbma_pm_job_id :{}",
                    ERROR_PREFIX,
                    err
                )
            })?;
    }

    let poll_historical_rates_job_id = &poll_historical_rates_job.guid();
    scheduler
        .add(poll_historical_rates_job)
        .await
        .map_err(|err| {
            anyhow!(
                "{} failed adding poll_historical_rates_job: {}",
                ERROR_PREFIX,
                err
            )
        })?;

    if !cron_cfg.cron_enable_poll_historical_rates {
        scheduler
            .remove(poll_historical_rates_job_id)
            .await
            .map_err(|err| {
                anyhow!(
                    "{} failed removing poll_historical_rates_job :{}",
                    ERROR_PREFIX,
                    err
                )
            })?;
    }

    Ok(scheduler)
}

//////////////////////////////////////////// HANDLERS AND JOBS ////////////////////////////////////////////
// Job::new_async adds job using UTC offset
async fn poll_latest_rates_handler(fx: impl ForexRates, fs: impl ForexStorage, base: Currencies) {
    let _ = forex::poll_rates(&fx, &fs, base).await;
}

fn poll_latest_rates_sge_job(
    cron_cfg: &Config,
    core_cfg: &'static pfm_core::global::Config,
) -> Result<Job, anyhow::Error> {
    Job::new_async(&cron_cfg.cron_tab_poll_rates_sge, |_uuid, _lock| {
        let forex = dep_forex_impl();
        let storage = dep_storage_impl();
        Box::pin(poll_latest_rates_handler(
            forex,
            storage,
            global::BASE_CURRENCY,
        ))
    })
    .map_err(|err| {
        anyhow!(
            "{} failed adding poll_latest_rates_handler: {}",
            ERROR_PREFIX,
            err
        )
    })
}

fn poll_latest_rates_lbma_am_job(
    cron_cfg: &Config,
    core_cfg: &'static pfm_core::global::Config,
) -> Result<Job, anyhow::Error> {
    Job::new_async(&cron_cfg.cron_tab_poll_rates_lbma_am, |_uuid, _lock| {
        let forex = dep_forex_impl();
        let storage = dep_storage_impl();
        Box::pin(poll_latest_rates_handler(
            forex,
            storage,
            global::BASE_CURRENCY,
        ))
    })
    .map_err(|err| {
        anyhow!(
            "{} failed adding poll_latest_rates_handler: {}",
            ERROR_PREFIX,
            err
        )
    })
}

fn poll_latest_rates_lbma_pm_job(
    cron_cfg: &Config,
    core_cfg: &'static pfm_core::global::Config,
) -> Result<Job, anyhow::Error> {
    Job::new_async(&cron_cfg.cron_tab_poll_rates_lbma_pm, |_uuid, _lock| {
        let forex = dep_forex_impl();
        let storage = dep_storage_impl();
        Box::pin(poll_latest_rates_handler(
            forex,
            storage,
            global::BASE_CURRENCY,
        ))
    })
    .map_err(|err| {
        anyhow!(
            "{} failed adding poll_latest_rates_handler: {}",
            ERROR_PREFIX,
            err
        )
    })
}

async fn poll_historical_rates_handler(
    fx: impl ForexHistoricalRates,
    fs: impl ForexStorage,
    date: DateTime<Utc>,
    base: Currencies,
) {
    let _ = forex::poll_historical_rates(&fx, &fs, date, base).await;
}

fn poll_historical_rates_job(
    cron_cfg: &Config,
    core_cfg: &'static pfm_core::global::Config,
) -> Result<Job, anyhow::Error> {
    Job::new_async(&cron_cfg.cron_tab_poll_historical_rates, |_uuid, _lock| {
        let forex = dep_forex_impl();
        let storage = dep_storage_impl();
        // TODO set time accordingly
        let date = Utc::now(); // it runs everytime this cron job invoked.

        Box::pin(poll_historical_rates_handler(
            forex,
            storage,
            date,
            global::BASE_CURRENCY,
        ))
    })
    .map_err(|err| {
        anyhow!(
            "{} failed adding poll_historical_rates_handler: {}",
            ERROR_PREFIX,
            err
        )
    })
}

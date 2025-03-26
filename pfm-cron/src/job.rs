use crate::Config;
use anyhow::{Context, Result};
use chrono::{DateTime, TimeDelta, Utc};
use pfm_core::{
    forex::{
        self, interface::ForexHistoricalRates, interface::ForexRates, interface::ForexStorage,
        Currency,
    },
    global,
};
use tokio_cron_scheduler::{Job, JobScheduler};

/// ----------------------------- JOBS AND HANDLERS -----------------------------
/// run at every hour
pub(crate) async fn poll_latest_rates_job<'a, API, STORAGE>(
    scheduler: &'a JobScheduler,
    cron_cfg: &Config,
    forex_api: API,
    forex_storage: STORAGE,
) -> Result<&'a JobScheduler, anyhow::Error>
where
    API: ForexRates + Clone + Send + Sync + 'static,
    STORAGE: ForexStorage + Clone + Send + Sync + 'static,
{
    let latest_rates_job = Job::new_async(&cron_cfg.crontab_poll_rates, move |_uuid, _lock| {
        Box::pin(poll_latest_rates_handler(
            forex_api.clone(),
            forex_storage.clone(),
            global::BASE_CURRENCY,
        ))
    })
    .context("cron creating poll_latest_rates_job")?;

    let latest_rates_job_id = latest_rates_job.guid();
    if !cron_cfg.cron_enable_poll_rates {
        scheduler
            .remove(&latest_rates_job_id)
            .await
            .context("cron removing poll_latest_rates_job")?;
        return Ok(scheduler);
    }

    scheduler
        .add(latest_rates_job)
        .await
        .context("cron registering poll_latest_rates_job")?;
    Ok(scheduler)
}

async fn poll_latest_rates_handler(fx: impl ForexRates, fs: impl ForexStorage, base: Currency) {
    let _ = forex::service::poll_rates(&fx, &fs, base).await;
}

/// run at every 01:00 AM UTC
pub(crate) async fn poll_historical_rates_job<'a, API, STORAGE>(
    scheduler: &'a JobScheduler,
    cron_cfg: &Config,
    forex_api: API,
    forex_storage: STORAGE,
) -> Result<&'a JobScheduler, anyhow::Error>
where
    API: ForexHistoricalRates + Clone + Send + Sync + 'static,
    STORAGE: ForexStorage + Clone + Send + Sync + 'static,
{
    let historical_rates_job = Job::new_async(
        &cron_cfg.crontab_poll_historical_rates,
        move |_uuid, _lock| {
            // everytime this cron run, pull data from yesterday
            let date = Utc::now() - TimeDelta::days(1);

            Box::pin(poll_historical_rates_handler(
                forex_api.clone(),
                forex_storage.clone(),
                date,
                global::BASE_CURRENCY,
            ))
        },
    )
    .context("cron creating poll_historical_rates_job job")?;

    let historical_rates_job_id = historical_rates_job.guid();
    if !cron_cfg.cron_enable_poll_historical_rates {
        scheduler
            .remove(&historical_rates_job_id)
            .await
            .context("cron removing poll_historical_rates_job")?;
        return Ok(scheduler);
    }

    scheduler
        .add(historical_rates_job)
        .await
        .context("cron registering poll_historical_rates_job")?;
    Ok(scheduler)
}

async fn poll_historical_rates_handler(
    fx: impl ForexHistoricalRates,
    fs: impl ForexStorage,
    date: DateTime<Utc>,
    base: Currency,
) {
    let _ = forex::service::poll_historical_rates(&fx, &fs, date, base).await;
}
// ----------------------------- END -----------------------------

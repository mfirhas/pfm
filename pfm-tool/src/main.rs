use chrono::{DateTime, Datelike, TimeDelta, TimeZone, Utc, Weekday};
use pfm_core::forex::interface::{ForexHistoricalRates, ForexStorage};
use pfm_core::forex::{service, ForexError};
use pfm_core::forex_impl::forex_storage::ForexStorageImpl;
use pfm_core::global;
use pfm_core::{
    forex::ForexResult, forex_impl::currency_api::Api as CurrencyAPI,
    forex_impl::exchange_api::Api as ExchangeAPI,
    forex_impl::open_exchange_api::Api as OpenExchangeRatesAPI,
};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let storage = ForexStorageImpl::new(global::storage_fs());
    let latest_historical =
        ForexStorage::get_historical_list(&storage, 1, 1, pfm_core::forex::entity::Order::DESC)
            .await
            .unwrap();
    let start_date = {
        if !latest_historical.rates_list.is_empty() {
            latest_historical.rates_list[0].data.date
        } else {
            Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap()
        }
    };
    println!("{}", start_date);

    let from = start_date;
    tokio::time::sleep(Duration::from_secs(5)).await;
    // let from = Utc.with_ymd_and_hms(2003, 9, 29, 0, 0, 0).unwrap();
    let to = Utc.with_ymd_and_hms(2024, 12, 31, 0, 0, 0).unwrap();
    let storage = ForexStorageImpl::new(global::storage_fs());
    let apiname = ApisName::OpenExchangeRatesAPI;
    let forex_api = select_api(apiname);
    let ret = fetch_historical_data(forex_api, storage, from, to).await;
    println!("{:?}", ret);
}

fn select_api(apiname: ApisName) -> Apis {
    match apiname {
        ApisName::ExchangeAPI => {
            unimplemented!()
        }
        ApisName::CurrencyAPI => {
            let currency_api = CurrencyAPI::new(
                &global::config().forex_currency_api_key,
                global::http_client(),
            );
            Apis::CurrencyAPI(currency_api)
        }
        ApisName::OpenExchangeRatesAPI => {
            let oxrapi = OpenExchangeRatesAPI::new(
                &global::config().forex_open_exchange_api_key,
                global::http_client(),
            );
            Apis::OpenExchangeRatesAPI(oxrapi)
        }
    }
}

pub enum ApisName {
    ExchangeAPI,
    CurrencyAPI,
    OpenExchangeRatesAPI,
}

#[derive(Clone)]
pub enum Apis {
    ExchangeAPI(ExchangeAPI),
    CurrencyAPI(CurrencyAPI),
    OpenExchangeRatesAPI(OpenExchangeRatesAPI),
}

fn weekdays(from: DateTime<Utc>, to: DateTime<Utc>) -> Vec<DateTime<Utc>> {
    let start_date = from;
    let end_date = to;
    let mut dates: Vec<DateTime<Utc>> = vec![];

    let mut current_date = start_date;
    while current_date <= end_date {
        let day = current_date.weekday();
        if day != Weekday::Sat && day != Weekday::Sun {
            dates.push(current_date);
        }
        if let Some(d) = current_date.checked_add_signed(TimeDelta::days(1)) {
            current_date = d;
        } else {
            break;
        }
    }

    dates
}

async fn fetch_historical_data(
    api: Apis,
    storage: ForexStorageImpl,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> ForexResult<()> {
    match api {
        Apis::OpenExchangeRatesAPI(api) => {
            let quota_remaining = api.status().await?.data.usage.requests_remaining;
            // 4 reqs/sec
            let rate_limit = 4;
            let seconds_per_batch = 1;
            let ret = fetch_historical_rates_data(
                api,
                storage,
                from,
                to,
                quota_remaining,
                rate_limit,
                seconds_per_batch,
            )
            .await;
            ret
        }
        Apis::CurrencyAPI(api) => {
            let quota_remaining = api.status().await?.quotas.month.remaining;
            // 9 reqs/minute
            let rate_limit = 10;
            let seconds_per_batch = 62;
            let ret = fetch_historical_rates_data(
                api,
                storage,
                from,
                to,
                quota_remaining,
                rate_limit,
                seconds_per_batch,
            )
            .await;
            ret
        }
        _ => return Err(ForexError::internal_error("not implemented yet")),
    }
}

async fn fetch_historical_rates_data<A, S>(
    forex_api: A,
    storage: S,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
    quota_remaining: u32,
    rate_limit: u32,
    seconds_per_batch: u32,
) -> ForexResult<()>
where
    A: ForexHistoricalRates + Clone + Send + Sync + 'static,
    S: ForexStorage + Clone + Send + Sync + 'static,
{
    let weekdays = weekdays(from, to);
    if quota_remaining <= 0 {
        return Err(ForexError::internal_error("no quota remained"));
    }

    let total_requests = std::cmp::min(weekdays.len() as u32, quota_remaining);
    let sleep = seconds_per_batch as u64;

    let mut completed_requests = 0;
    let mut weekday_index = 0;

    while completed_requests < total_requests && weekday_index < weekdays.len() {
        let batch_size = std::cmp::min(total_requests - completed_requests, rate_limit);
        let batch_size = std::cmp::min(batch_size, weekdays.len() as u32 - weekday_index as u32);

        let mut handles = Vec::new();
        for _ in 0..batch_size {
            let api_clone = forex_api.clone();
            let storage_clone = storage.clone();
            let date = weekdays[weekday_index];
            let index = weekday_index;

            handles.push(tokio::spawn(async move {
                let ret = service::poll_historical_rates(
                    &api_clone,
                    &storage_clone,
                    date,
                    global::BASE_CURRENCY,
                )
                .await;
                println!("{}. Result date {}: {:?}", index, date, ret);
            }));
            weekday_index += 1;
        }

        for handle in handles {
            let _ = handle.await;
        }

        completed_requests += batch_size;

        tokio::time::sleep(Duration::from_secs(sleep)).await;
    }

    let latest = storage
        .get_historical_list(1, 1, pfm_core::forex::entity::Order::DESC)
        .await?;
    if !latest.rates_list.is_empty() {
        println!(
            "fetch_historical_data latest date fetched: {}",
            latest.rates_list[0].data.date
        );
    } else {
        println!("no data fetched");
    }

    Ok(())
}

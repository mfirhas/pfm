use chrono::{DateTime, Datelike, TimeDelta, TimeZone, Utc, Weekday};
use pfm_core::forex::interface::ForexStorage;
use pfm_core::forex::{service, ForexError};
use pfm_core::forex_impl::forex_storage::ForexStorageImpl;
use pfm_core::forex_impl::open_exchange_api;
use pfm_core::global;
use pfm_core::{
    forex::ForexResult, forex_impl::currency_api::Api as CurrencyAPI,
    forex_impl::exchange_api::Api as ExchangeAPI,
    forex_impl::open_exchange_api::Api as OpenExchangeRatesAPI,
};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    let from = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
    let to = Utc.with_ymd_and_hms(2024, 12, 31, 0, 0, 0).unwrap();
    // println!("{:?} {}", weekdays(from, to), weekdays(from, to).len());
    let oxrapi = open_exchange_api::Api::new(
        &global::config().forex_open_exchange_api_key,
        global::http_client(),
    );
    let storage = ForexStorageImpl::new(global::storage_fs());
    let ret = fetch_historical_data(Apis::OpenExchangeRatesAPI(oxrapi), storage, from, to).await;
    println!("{:?}", ret);
}

#[derive(Clone)]
pub enum Apis {
    ExchangeAPI(ExchangeAPI), // rate limit 5 reqs/sec
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
            let weekdays = weekdays(from, to);
            let quota_remaining = api.status().await?.data.usage.requests_remaining;
            if quota_remaining <= 0 {
                return Err(ForexError::internal_error("no quota remained"));
            } else {
                let total_requests = std::cmp::min(weekdays.len() as u32, quota_remaining);
                let rate_limit = 4; // Requests per second

                let mut completed_requests = 0;

                let mut weekday_index = 0 as usize;

                while completed_requests < total_requests {
                    let batch_size = (total_requests - completed_requests).min(rate_limit);

                    let mut handles = Vec::new();
                    for _ in 0..batch_size {
                        let api_ref = api.clone();
                        let storage_ref = storage.clone();
                        let date = weekdays[weekday_index];
                        handles.push(tokio::spawn(async move {
                            let ret = service::poll_historical_rates(
                                &api_ref,
                                &storage_ref,
                                date,
                                global::BASE_CURRENCY,
                            )
                            .await;
                            println!("{}. Result date {}: {:?}", weekday_index, date, ret);
                        }));
                        weekday_index += 1;
                    }

                    for handle in handles {
                        let _ = handle.await;
                    }

                    completed_requests += batch_size;

                    sleep(Duration::from_secs(1)).await;
                }
            }

            let latest = storage
                .get_historical_list(1, 1, pfm_core::forex::entity::Order::DESC)
                .await?;
            if latest.rates_list.len() > 0 {
                println!(
                    "fetch_historical_data latest date fetched: {}",
                    latest.rates_list[0].data.date
                );
            } else {
                println!("no data fetched");
            }
        }
        _ => return Err(ForexError::internal_error("not implemented yet")),
    }

    Ok(())
}

use chrono::{DateTime, Datelike, Months, TimeDelta, TimeZone, Timelike, Utc, Weekday};
use pfm_core::forex::interface::{ForexHistoricalRates, ForexStorage, ForexTimeseriesRates};
use pfm_core::forex::{service, Currency, ForexError, Money};
use pfm_core::forex_impl::forex_storage::ForexStorageImpl;
use pfm_core::global;
use pfm_core::{
    forex::ForexResult, forex_impl::currency_api::Api as CurrencyAPI,
    forex_impl::currencybeacon::Api as CurrencyBeaconAPI,
    forex_impl::exchange_api::Api as ExchangeAPI,
    forex_impl::open_exchange_api::Api as OpenExchangeRatesAPI,
};
use rust_decimal_macros::dec;
use sha2::Digest;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[tokio::main]
async fn main() {
    // fetch historical data to populate historical data split into its rate limit
    // do_fetch_historical_data().await;

    // fetch timeseries data and store them
    // do_fetch_timeseries_and_store().await;

    // read csv data of crypto prices
    // do_update_crypto_data().await;

    // calculate checksum for files inside pfm/pfm-data/historical/historical-...json and store the checksums into pfm/checksums/...
    // do_calculate_and_store_checksum();

    // check checksum
    // do_compare_checksums();
}

async fn do_fetch_historical_data() {
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
    let to = Utc.with_ymd_and_hms(2025, 3, 30, 23, 59, 59).unwrap();
    let storage = ForexStorageImpl::new(global::storage_fs());
    let apiname = ApisName::CurrencyBeaconAPI;
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
        ApisName::CurrencyBeaconAPI => {
            let currencybeaconapi = CurrencyBeaconAPI::new(
                &global::config().forex_currencybeacon_api_key,
                global::http_client(),
            );
            Apis::CurrencyBeacon(currencybeaconapi)
        }
    }
}

pub enum ApisName {
    ExchangeAPI,
    CurrencyAPI,
    OpenExchangeRatesAPI,
    CurrencyBeaconAPI,
}

#[derive(Clone)]
pub enum Apis {
    ExchangeAPI(ExchangeAPI),
    CurrencyAPI(CurrencyAPI),
    OpenExchangeRatesAPI(OpenExchangeRatesAPI),
    CurrencyBeacon(CurrencyBeaconAPI),
}

fn alldays(from: DateTime<Utc>, to: DateTime<Utc>) -> Vec<DateTime<Utc>> {
    let start_date = from;
    let end_date = to;
    let mut dates: Vec<DateTime<Utc>> = vec![];

    let mut current_date = start_date;
    while current_date <= end_date {
        dates.push(current_date);
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
        Apis::CurrencyBeacon(api) => {
            let quota_remaining = 1000 as u32;
            let rate_limit = 5;
            let seconds_per_batch = 5;
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
    let alldays = alldays(from, to);
    if quota_remaining <= 0 {
        return Err(ForexError::internal_error("no quota remained"));
    }

    let total_requests = std::cmp::min(alldays.len() as u32, quota_remaining);
    let sleep = seconds_per_batch as u64;

    let mut completed_requests = 0;
    let mut allday_index = 0;

    while completed_requests < total_requests && allday_index < alldays.len() {
        let batch_size = std::cmp::min(total_requests - completed_requests, rate_limit);
        let batch_size = std::cmp::min(batch_size, alldays.len() as u32 - allday_index as u32);

        let mut handles = Vec::new();
        for _ in 0..batch_size {
            let api_clone = forex_api.clone();
            let storage_clone = storage.clone();
            let date = alldays[allday_index];
            let index = allday_index;

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
            allday_index += 1;
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

async fn do_fetch_timeseries_and_store() {
    let start_date = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
    let end_date = Utc.with_ymd_and_hms(2025, 3, 24, 23, 59, 59).unwrap();
    // let end_date = Utc.with_ymd_and_hms(1999, 12, 31, 23, 59, 59).unwrap();
    let ranges = split_date_range_yearly(start_date, end_date, 5);
    for range in ranges {
        let from = range.0;
        let to = range.1;
        println!("fetching historical data from {} till {}", from, to);
        fetch_timeseries_and_store(from, to).await;
        tokio::time::sleep(Duration::from_secs(62)).await;
    }
}

async fn fetch_timeseries_and_store(start_date: DateTime<Utc>, end_date: DateTime<Utc>) {
    let storage_impl = ForexStorageImpl::new(global::storage_fs());
    let forex_api = CurrencyBeaconAPI::new(
        &global::config().forex_currencybeacon_api_key,
        global::http_client(),
    );
    let ret = forex_api
        .timeseries_rates(start_date, end_date, global::BASE_CURRENCY)
        .await;
    let rates = ret.unwrap();
    let stored = ForexStorage::insert_historical_batch(&storage_impl, rates).await;
    dbg!(&stored);
    stored.unwrap();
}

fn split_date_range_yearly(
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    max_years: u32,
) -> Vec<(DateTime<Utc>, DateTime<Utc>)> {
    let mut ranges = Vec::new();
    let max_months = Months::new(max_years * 12);

    let mut current_start = start_date;

    while current_start < end_date {
        // Calculate end date by adding max_months (but subtract 1 second to avoid overlap)
        let mut current_end = (current_start + max_months) - Duration::from_secs(1);

        // Don't let the end date exceed the original end_date
        if current_end > end_date {
            current_end = end_date;
        }

        ranges.push((current_start, current_end));

        // Move to the next second after current_end for the next range
        current_start = current_end + Duration::from_secs(1);
    }

    ranges
}

// csv parser
// Parsing data from coinmarketcap.com
// This will parse the rates data from csv and convert into (date, Vec<Money>)
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufReader, Write};

#[derive(Debug, Deserialize)]
struct CryptoRecord {
    #[serde(rename = "timeOpen")]
    time_open: String,
    #[serde(rename = "timeClose")]
    time_close: String,
    #[serde(rename = "timeHigh")]
    time_high: String,
    #[serde(rename = "timeLow")]
    time_low: String,
    name: String,
    open: Decimal,
    high: Decimal,
    low: Decimal,
    close: Decimal,
    volume: Decimal,
    marketCap: Decimal,
    timestamp: String,
}

fn read_csv(
    currency: Currency,
    file_path: &str,
) -> Result<HashMap<(Currency, DateTime<Utc>), Decimal>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(BufReader::new(file));

    let mut ret: Vec<((Currency, DateTime<Utc>), Decimal)> = vec![];
    for result in rdr.deserialize() {
        let record: CryptoRecord = result?;
        let date = record
            .timestamp
            .parse::<DateTime<Utc>>()?
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string()
            .parse()?;
        let usd_crypto_rate = dec!(1).checked_div(record.close).unwrap_or_default();
        let data = ((currency, date), usd_crypto_rate);
        ret.push(data);
    }

    let map = ret.into_iter().collect();
    Ok(map)
}

fn iterate_and_parse(
    currency: Currency,
    dir: impl AsRef<Path>,
) -> Result<HashMap<(Currency, DateTime<Utc>), Decimal>, Box<dyn Error>> {
    let mut aggregated_data = HashMap::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            match read_csv(currency, path.to_str().ok_or("Invalid path")?) {
                Ok(data) => aggregated_data.extend(data),
                Err(e) => eprintln!("Failed to read {:?}: {}", path, e),
            }
        }
    }

    Ok(aggregated_data)
}

async fn do_update_crypto_data() {
    let forex_storage = ForexStorageImpl::new(global::storage_fs());
    let start_date = Utc.with_ymd_and_hms(2010, 1, 1, 0, 0, 0).unwrap();
    let end_date = Utc.with_ymd_and_hms(2025, 4, 10, 23, 59, 59).unwrap();

    let csv_btc = (
        Currency::BTC,
        "/Users/mfirhas/pfm_backup/crypto_prices_history/btc",
    );
    let csv_eth = (
        Currency::ETH,
        "/Users/mfirhas/pfm_backup/crypto_prices_history/eth",
    );
    let csv_sol = (
        Currency::SOL,
        "/Users/mfirhas/pfm_backup/crypto_prices_history/sol",
    );
    let csv_xrp = (
        Currency::XRP,
        "/Users/mfirhas/pfm_backup/crypto_prices_history/xrp",
    );
    let csv_ada = (
        Currency::ADA,
        "/Users/mfirhas/pfm_backup/crypto_prices_history/ada",
    );

    let mut crypto_data = HashMap::new();
    let btc_data = iterate_and_parse(csv_btc.0, csv_btc.1).unwrap();
    let eth_data = iterate_and_parse(csv_eth.0, csv_eth.1).unwrap();
    let sol_data = iterate_and_parse(csv_sol.0, csv_sol.1).unwrap();
    let xrp_data = iterate_and_parse(csv_xrp.0, csv_xrp.1).unwrap();
    let ada_data = iterate_and_parse(csv_ada.0, csv_ada.1).unwrap();
    crypto_data.extend(btc_data);
    crypto_data.extend(eth_data);
    crypto_data.extend(sol_data);
    crypto_data.extend(xrp_data);
    crypto_data.extend(ada_data);

    let mut ret = ForexStorage::get_historical_range(&forex_storage, start_date, end_date)
        .await
        .unwrap();
    for rate in ret.iter_mut() {
        if rate.data.rates.btc.is_zero() {
            rate.data.rates.btc = *crypto_data
                .get(&(Currency::BTC, rate.data.date))
                .unwrap_or(&dec!(0));
        }

        if rate.data.rates.eth.is_zero() {
            rate.data.rates.eth = *crypto_data
                .get(&(Currency::ETH, rate.data.date))
                .unwrap_or(&dec!(0));
        }

        if rate.data.rates.sol.is_zero() {
            rate.data.rates.sol = *crypto_data
                .get(&(Currency::SOL, rate.data.date))
                .unwrap_or(&dec!(0));
        }

        if rate.data.rates.xrp.is_zero() {
            rate.data.rates.xrp = *crypto_data
                .get(&(Currency::XRP, rate.data.date))
                .unwrap_or(&dec!(0));
        }

        if rate.data.rates.ada.is_zero() {
            rate.data.rates.ada = *crypto_data
                .get(&(Currency::ADA, rate.data.date))
                .unwrap_or(&dec!(0));
        }

        ForexStorage::insert_historical(&forex_storage, rate.data.date, &rate)
            .await
            .unwrap();
    }
}

fn do_calculate_and_store_checksum() {
    let pfm_data_historical_path = "/Users/mfirhas/pfm/pfm-data/historical";
    let historical_dir = PathBuf::from(pfm_data_historical_path);
    let pfm_data_checksums_path = "/Users/mfirhas/pfm/checksums/historical";
    let checksums_dir = PathBuf::from(pfm_data_checksums_path);
    let mut entries = std::fs::read_dir(historical_dir).unwrap();
    let mut index = 0;
    // loop the historical dir
    while let Some(entry) = entries.next() {
        let year_dir = entry.unwrap().path();
        if !year_dir.is_dir() && !is_ds_store(&year_dir) {
            panic!("{:?} is not a directory", &year_dir.as_path());
        }
        if is_ds_store(&year_dir) {
            continue;
        }

        // loop the year dir
        let mut year_dir_entries = std::fs::read_dir(&year_dir).unwrap();
        while let Some(files) = year_dir_entries.next() {
            let file = files.unwrap().path();
            if !file.is_file() && !is_ds_store(&file) {
                panic!("{:?} is not a file", &file.as_path());
            }
            if is_ds_store(&file) {
                continue;
            }
            // generate checksum
            let checksum = generate_checksum(&file).unwrap();
            let checksum_filename = format!(
                "{}_checksum.json",
                &file.file_stem().unwrap().to_string_lossy().to_string()
            );
            let checksum_file_dir_path =
                checksums_dir.join(&year_dir.file_stem().unwrap().to_string_lossy().to_string());
            std::fs::create_dir_all(&checksum_file_dir_path).unwrap();
            let checksum_file_path = checksum_file_dir_path.join(checksum_filename);
            let checksum_data = ChecksumData {
                checksum_date: Utc::now().with_nanosecond(0).unwrap(),
                file: file.file_name().unwrap().to_string_lossy().to_string(),
                checksum,
            };

            println!(
                "{}. Checksum Data: {:?}, \nChecksum filepath: {:?}",
                index,
                &checksum_data,
                &checksum_file_path.as_path()
            );

            // store the checksum data
            let string_data = serde_json::to_string_pretty(&checksum_data).unwrap();
            std::fs::File::create(&checksum_file_path)
                .unwrap()
                .write_all(string_data.as_bytes())
                .unwrap();

            index += 1;
        }
    }
    println!("Total files processed: {}", index);
}

fn is_ds_store(path: &PathBuf) -> bool {
    path.file_name()
        .map(|name| name == ".DS_Store")
        .unwrap_or(false)
}

#[derive(Debug, Serialize, Deserialize)]
struct ChecksumData {
    checksum_date: DateTime<Utc>,
    file: String,
    checksum: String,
}

fn generate_checksum(path: impl AsRef<Path>) -> Result<String, std::io::Error> {
    let data = fs::read(path)?;
    let mut hasher = sha2::Sha256::new();
    hasher.update(&data);
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}

fn do_compare_checksums() {
    let new_checksum_path = PathBuf::from("/Users/mfirhas/pfm/checksums/historical");
    let prev_checksum_path = PathBuf::from("/Users/mfirhas/pfm_backup/pfm/checksums/historical");
    let ret = compare_checksums(&new_checksum_path, &prev_checksum_path);
    println!("Result: {:?}", &ret);
    if ret.is_empty() {
        println!(
            "All new checksums in {:?} are equal to previous checksums in {:?}",
            &new_checksum_path.as_path(),
            &prev_checksum_path.as_path()
        );
    } else {
        println!(
            "inequals: {} \nStarts from: {}, \nEnded at: {}",
            &ret.len(),
            &ret[0],
            &ret[&ret.len() - 1]
        );
    }
}

fn compare_checksums(new_checksums: &PathBuf, prev_checksums: &PathBuf) -> Vec<String> {
    // contains path of different checksums
    let mut results: Vec<String> = vec![];
    let mut new_checksums_entries = std::fs::read_dir(new_checksums).unwrap();
    let mut index = 0;
    while let Some(year_dir) = new_checksums_entries.next() {
        let year_dir_path = year_dir.unwrap().path();
        if !year_dir_path.is_dir() {
            panic!("{:?} is not a directory", &year_dir_path.as_path());
        }
        let mut year_dir_entries = std::fs::read_dir(&year_dir_path).unwrap();
        while let Some(file) = year_dir_entries.next() {
            let file_path = file.unwrap().path();
            if !file_path.is_file() {
                panic!("not a file");
            }
            let content = std::fs::read_to_string(&file_path).unwrap();
            let checksum_data: ChecksumData = serde_json::from_str(&content).unwrap();

            let prev_checksum_file_path = prev_checksums
                .join(&year_dir_path.file_stem().unwrap().to_str().unwrap())
                .join(&file_path.file_name().unwrap().to_str().unwrap());
            // println!("-->{:?}", &prev_checksum_file_path.as_path());
            let prev_content = std::fs::read_to_string(&prev_checksum_file_path).unwrap();
            let prev_checksum_data: ChecksumData = serde_json::from_str(&prev_content).unwrap();

            println!(
                "Comparing {:?} \nwith \n{:?}",
                &file_path.as_path(),
                &prev_checksum_file_path.as_path()
            );

            println!(
                "New checksum: {:?} \nPrev checksum: {:?}",
                &checksum_data, &prev_checksum_data
            );
            if checksum_data.checksum != prev_checksum_data.checksum {
                results.push(file_path.to_string_lossy().to_string());
            }

            index += 1;
        }
    }

    results.sort();

    println!("Total checked: {}", index);

    results
}

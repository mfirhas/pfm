// forex_storage.rs implement storage mechanism for SERVER side http and cron.
// implementations for database to store forex data polled from the APIs.
// using filesystem with tokio

use std::fmt::Debug;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use crate::error::AsInternalError;
use crate::forex::entity::{HistoricalRates, Order, Rates, RatesList, RatesResponse};
use crate::forex::interface::{ForexStorage, ForexStorageDeletion};
use crate::forex::ForexResult;
use crate::forex::{ForexError, Money};
use crate::global::StorageFS;
use anyhow::Context;
use async_trait::async_trait;
use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use serde::{Deserialize, Serialize};
use tokio::fs::{self, read_dir, File};
use tokio::io::AsyncWriteExt;
use tracing::instrument;

const ERROR_PREFIX: &str = "[FOREX][storage_impl]";

const LATEST_FILENAME_FORMAT: &str = "latest-{YYYY}-{MM}-{DD}T{hh}:{mm}:{ss}Z.json";

const HISTORICAL_FILENAME_FORMAT: &str = "historical-{YYYY}-{MM}-{DD}Z.json";

const FILE_PERMISSION: u32 = 0o640;

#[derive(Clone)]
pub struct ForexStorageImpl {
    fs: StorageFS,
}

impl ForexStorageImpl {
    pub fn new(fs: StorageFS) -> Self {
        Self { fs }
    }

    async fn set_permission(pathbuf: &PathBuf) -> ForexResult<()> {
        // Set permissions to 640 (owner read/write only)
        let mut perms = fs::metadata(&pathbuf)
            .await
            .context("forex storage read metadata")
            .as_internal_err()?
            .permissions();
        perms.set_mode(FILE_PERMISSION);
        fs::set_permissions(&pathbuf, perms)
            .await
            .context("forex storage setting permission")
            .as_internal_err()?;

        Ok(())
    }

    async fn insert_latest<T>(
        &self,
        date: DateTime<Utc>,
        rates: &RatesResponse<T>,
    ) -> ForexResult<()>
    where
        T: Debug + Serialize + for<'de> Deserialize<'de> + Send + Sync,
    {
        let json_string = serde_json::to_string_pretty(&rates)
            .context("forex storage insert latest parse into json string")
            .as_internal_err()?;

        let latest_write = self.fs.write().await;
        let latest_write = latest_write.latest();
        let latest_write = latest_write.join(generate_latest_file_path(date));

        let mut file = File::create(&latest_write)
            .await
            .context("forex storage insert latest create path")
            .as_internal_err()?;
        file.write_all(json_string.as_bytes())
            .await
            .context("forex storage insert latest write")
            .as_internal_err()?;
        file.flush()
            .await
            .context("forex storage insert latest flush")
            .as_internal_err()?;

        Self::set_permission(&latest_write).await?;

        Ok(())
    }

    #[instrument(skip(self), ret)]
    async fn get_latest(&self) -> ForexResult<RatesResponse<Rates>> {
        let latest_read = self.fs.read().await;
        let latest_read = latest_read.latest();

        let mut entries = fs::read_dir(latest_read)
            .await
            .context("storage get latest read dir")
            .as_internal_err()?;

        let mut files: Vec<PathBuf> = Vec::new();
        while let Some(entry) = entries
            .next_entry()
            .await
            .context("storage get latest reading entries")
            .as_internal_err()?
        {
            let path = entry.path();
            files.push(path);
        }

        if files.is_empty() {
            return Err(ForexError::internal_error("storage get latest dir empty"));
        }

        // sort descending
        files.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

        let content = fs::read_to_string(&files[0])
            .await
            .context("storage get latest reading content")
            .as_internal_err()?;

        let rates: RatesResponse<Rates> = serde_json::from_str(&content)
            .context("storage get latest parse to json")
            .as_internal_err()?;

        Ok(rates)
    }

    async fn insert_historical<T>(
        &self,
        date: DateTime<Utc>,
        rates: &RatesResponse<T>,
    ) -> ForexResult<()>
    where
        T: Debug + Serialize + for<'de> Deserialize<'de> + Send + Sync,
    {
        let json_string = serde_json::to_string_pretty(&rates)
            .context("storage insert historical parse input into json string")
            .as_internal_err()?;

        let historical_write = self.fs.write().await;
        let historical_write = historical_write.historical();
        let historical_write = historical_write.join(generate_historical_file_path(date));

        let year_dir = historical_write.parent();
        if let Some(dir) = year_dir {
            if !dir.is_dir() {
                tokio::fs::create_dir_all(dir)
                    .await
                    .context("storage insert historical create year dir")
                    .as_internal_err()?;
            }
        } else {
            return Err(ForexError::internal_error(
                "storage insert historical create year dir",
            ));
        };

        let mut file = File::create(&historical_write)
            .await
            .context("storage insert historical create filepath")
            .as_internal_err()?;
        file.write_all(json_string.as_bytes())
            .await
            .context("storage insert historical write content")
            .as_internal_err()?;
        file.flush()
            .await
            .context("storage insert historical flush")
            .as_internal_err()?;

        Self::set_permission(&historical_write).await?;

        Ok(())
    }

    async fn insert_historical_batch(
        &self,
        rates: Vec<RatesResponse<HistoricalRates>>,
    ) -> ForexResult<()> {
        let historical_write = self.fs.write().await;
        let historical_write = historical_write.historical();

        for rate in rates {
            let date = rate.data.date;

            let file_full_path = historical_write.join(generate_historical_file_path(date));

            let year_dir = file_full_path.parent();
            if let Some(dir) = year_dir {
                if !dir.is_dir() {
                    tokio::fs::create_dir_all(dir)
                        .await
                        .context("storage insert historical batch create year dir")
                        .as_internal_err()?;
                }
            } else {
                return Err(ForexError::internal_error(
                    "storage insert historical batch create year dir",
                ));
            };

            let json_string = serde_json::to_string_pretty(&rate)
                .context("storage insert historical batch parse input into json string")
                .as_internal_err()?;

            let mut file = File::create(&file_full_path)
                .await
                .context("storage insert historical batch create filepath")
                .as_internal_err()?;
            file.write_all(json_string.as_bytes())
                .await
                .context("storage insert historical batch write content")
                .as_internal_err()?;
            file.flush()
                .await
                .context("storage insert historical batch flush")
                .as_internal_err()?;

            Self::set_permission(&file_full_path).await?;
        }

        Ok(())
    }

    async fn update_historical_rates_data(
        &self,
        date: DateTime<Utc>,
        new_rates: Vec<Money>,
    ) -> ForexResult<RatesResponse<HistoricalRates>> {
        let mut historical_rates = {
            let before_historical_rates = self
                .get_historical(date)
                .await
                .context("storage update historical get historical")
                .as_internal_err()?;
            before_historical_rates
        };

        for v in new_rates {
            match v {
                // fiat

                // north america
                Money::USD(value) => {
                    historical_rates.data.rates.usd = value;
                }
                Money::CAD(value) => {
                    historical_rates.data.rates.cad = value;
                }

                // europe
                Money::EUR(value) => {
                    historical_rates.data.rates.eur = value;
                }
                Money::GBP(value) => {
                    historical_rates.data.rates.gbp = value;
                }
                Money::CHF(value) => {
                    historical_rates.data.rates.chf = value;
                }
                Money::RUB(value) => {
                    historical_rates.data.rates.rub = value;
                }

                // east asia
                Money::CNY(value) => {
                    historical_rates.data.rates.cny = value;
                }
                Money::JPY(value) => {
                    historical_rates.data.rates.jpy = value;
                }
                Money::KRW(value) => {
                    historical_rates.data.rates.krw = value;
                }
                Money::HKD(value) => {
                    historical_rates.data.rates.hkd = value;
                }

                // south-east asia
                Money::IDR(value) => {
                    historical_rates.data.rates.idr = value;
                }
                Money::MYR(value) => {
                    historical_rates.data.rates.myr = value;
                }
                Money::SGD(value) => {
                    historical_rates.data.rates.sgd = value;
                }
                Money::THB(value) => {
                    historical_rates.data.rates.thb = value;
                }

                // middle-east
                Money::SAR(value) => {
                    historical_rates.data.rates.sar = value;
                }
                Money::AED(value) => {
                    historical_rates.data.rates.aed = value;
                }
                Money::KWD(value) => {
                    historical_rates.data.rates.kwd = value;
                }

                // south asia
                Money::INR(value) => {
                    historical_rates.data.rates.inr = value;
                }

                // apac
                Money::AUD(value) => {
                    historical_rates.data.rates.aud = value;
                }
                Money::NZD(value) => {
                    historical_rates.data.rates.nzd = value;
                }

                //// precious metals
                Money::XAU(value) => {
                    historical_rates.data.rates.xau = value;
                }
                Money::XAG(value) => {
                    historical_rates.data.rates.xag = value;
                }
                Money::XPT(value) => {
                    historical_rates.data.rates.xpt = value;
                }

                //// crypto
                Money::BTC(value) => {
                    historical_rates.data.rates.btc = value;
                }
                Money::ETH(value) => {
                    historical_rates.data.rates.eth = value;
                }
                Money::SOL(value) => {
                    historical_rates.data.rates.sol = value;
                }
                Money::XRP(value) => {
                    historical_rates.data.rates.xrp = value;
                }
                Money::ADA(value) => {
                    historical_rates.data.rates.ada = value;
                }
            }
        }

        let json_string = serde_json::to_string_pretty(&historical_rates)
            .context("storage update historical parse input into json string")
            .as_internal_err()?;

        let historical_write_guard = self.fs.write().await;
        let historical_write = historical_write_guard.historical();
        let historical_write = historical_write.join(generate_historical_file_path(date));

        let mut file = File::create(&historical_write)
            .await
            .context("storage update historical create filepath")
            .as_internal_err()?;
        file.write_all(json_string.as_bytes())
            .await
            .context("storage update historical write content")
            .as_internal_err()?;
        file.flush()
            .await
            .context("storage update historical flush")
            .as_internal_err()?;
        drop(historical_write_guard);

        let updated_historical_rates = self
            .get_historical(date)
            .await
            .context("storage update historical get historical")
            .as_internal_err()?;

        Ok(updated_historical_rates)
    }

    #[instrument(skip(self), ret)]
    async fn get_historical(
        &self,
        date: DateTime<Utc>,
    ) -> ForexResult<RatesResponse<HistoricalRates>> {
        let historical_read = self.fs.read().await;
        let historical_read = historical_read.historical();
        let filepath = historical_read.join(&generate_historical_file_path(date));

        let content = fs::read_to_string(&filepath)
            .await
            .context("storage get historical read file")
            .as_internal_err()?;

        let rates: RatesResponse<HistoricalRates> = serde_json::from_str(&content)
            .context("storage get historical parse to json")
            .as_internal_err()?;

        Ok(rates)
    }

    #[instrument(skip(self), ret)]
    async fn get_historical_range(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> ForexResult<Vec<RatesResponse<HistoricalRates>>> {
        let start_year = start_date.year();
        let end_year = end_date.year();

        let mut resp = vec![];

        let historical_read = self.fs.read().await;
        let historical_read_path = historical_read.historical();
        let mut entries = read_dir(historical_read_path)
            .await
            .context("get historical range reading historical path")
            .as_internal_err()?;
        while let Some(historical_entry) = entries
            .next_entry()
            .await
            .context("get historical range iterating over historical entries")
            .as_internal_err()?
        {
            let metadata = historical_entry
                .metadata()
                .await
                .context("get historical range reading entry metadata")
                .as_internal_err()?;
            if !metadata.is_dir() {
                return Err(ForexError::internal_error(
                    "some historical directory contents contain non directory",
                ));
            }
            let year_dir = historical_entry
                .file_name()
                .to_string_lossy()
                .trim()
                .parse::<i32>()
                .context("get historical range converting historical entry file name to year i32")
                .as_internal_err()?;

            // year on directory not within date range
            if year_dir < start_year || year_dir > end_year {
                continue;
            }

            let mut year_entries = read_dir(historical_entry.path())
                .await
                .context("get historical range reading historical subentry")
                .as_internal_err()?;
            while let Some(sub_historical_entry) = year_entries
                .next_entry()
                .await
                .context("get historical range iterating over historical sub entries")
                .as_internal_err()?
            {
                let sub_meta = sub_historical_entry
                    .metadata()
                    .await
                    .context("get historical range read sub meta")
                    .as_internal_err()?;
                if !sub_meta.is_file() {
                    return Err(ForexError::internal_error(
                        "some sub historical entries content are not files",
                    ));
                }
                let file_date: DateTime<Utc> = parse_historical_file_path(
                    sub_historical_entry.file_name().to_string_lossy().trim(),
                )
                .ok_or(ForexError::internal_error(
                    "get historical range parsing filename",
                ))?;

                if file_date < start_date || file_date > end_date {
                    continue;
                }

                // read the content of the file
                let content = fs::read_to_string(sub_historical_entry.path())
                    .await
                    .context("get historical range read file content")
                    .as_internal_err()?;
                let rates: RatesResponse<HistoricalRates> = serde_json::from_str(&content)
                    .context("get historical range parse content to json")
                    .as_internal_err()?;

                resp.push(rates);
            }
        }

        resp.sort_by_key(|v| v.data.date);

        Ok(resp)
    }

    async fn get_latest_list(
        &self,
        page: u32,
        size: u32,
        order: Order,
    ) -> ForexResult<RatesList<RatesResponse<Rates>>> {
        let latest_read = self.fs.read().await;
        let latest_read = latest_read.latest();

        let mut entries = fs::read_dir(latest_read)
            .await
            .context("storage get latest list read dir")
            .as_internal_err()?;

        let mut files: Vec<RatesResponse<Rates>> = Vec::new();
        while let Some(entry) = entries
            .next_entry()
            .await
            .context("storage get latest list read entries")
            .as_internal_err()?
        {
            let path = entry.path();
            let content = tokio::fs::read_to_string(&path)
                .await
                .context("storage get latest list reading file")
                .as_internal_err()?;
            let resp: RatesResponse<Rates> = serde_json::from_str(&content)
                .context("storage get latest list parse to json")
                .as_internal_err()?;
            files.push(resp);
        }

        if files.is_empty() {
            return Ok(RatesList {
                has_prev: false,
                rates_list: vec![],
                has_next: false,
            });
        }

        match order {
            Order::ASC => files.sort_by_key(|rate| rate.data.latest_update),
            Order::DESC => files.sort_by(|a, b| b.data.latest_update.cmp(&a.data.latest_update)),
        }

        let paginated = Self::paginate_rates_list(&files, page, size);

        let resp = RatesList {
            has_prev: paginated.has_prev,
            rates_list: paginated.rates_list,
            has_next: paginated.has_next,
        };

        Ok(resp)
    }

    async fn get_historical_list(
        &self,
        page: u32,
        size: u32,
        order: Order,
    ) -> ForexResult<RatesList<RatesResponse<HistoricalRates>>> {
        let historical_read = self.fs.read().await;
        let historical_read = historical_read.historical();

        let mut entries = fs::read_dir(historical_read)
            .await
            .context("storage get historical list read dir")
            .as_internal_err()?;

        let mut files: Vec<RatesResponse<HistoricalRates>> = Vec::new();
        while let Some(entry) = entries
            .next_entry()
            .await
            .context("storage get historical list reading entries")
            .as_internal_err()?
        {
            let path = entry.path();
            let mut sub_entries = fs::read_dir(&path)
                .await
                .context("storage get historical list read sub entry")
                .as_internal_err()?;
            while let Some(sub_entry) = sub_entries
                .next_entry()
                .await
                .context("storage get historical list read subentries")
                .as_internal_err()?
            {
                let sub_entry_path = sub_entry.path();
                let content = tokio::fs::read_to_string(&sub_entry_path)
                    .await
                    .context("storage get historical list read subentry content")
                    .as_internal_err()?;
                let resp: RatesResponse<HistoricalRates> = serde_json::from_str(&content)
                    .context("storage get historical list parse subentry to json")
                    .as_internal_err()?;
                files.push(resp);
            }
        }

        if files.is_empty() {
            return Ok(RatesList {
                has_prev: false,
                rates_list: vec![],
                has_next: false,
            });
        }

        match order {
            Order::ASC => files.sort_by_key(|rate| rate.data.date),
            Order::DESC => files.sort_by(|a, b| b.data.date.cmp(&a.data.date)),
        }

        let paginated = Self::paginate_rates_list(&files, page, size);

        let resp = RatesList {
            has_prev: paginated.has_prev,
            rates_list: paginated.rates_list,
            has_next: paginated.has_next,
        };

        Ok(resp)
    }

    // deletions impls
    async fn clear_latest(&self) -> ForexResult<()> {
        let latest_write = self.fs.write().await;
        let latest_write = latest_write.latest();

        let mut entries = fs::read_dir(latest_write)
            .await
            .context("storage clear latest read dir")
            .as_internal_err()?;
        let mut files = Vec::new();

        // Collect all files with filenames
        while let Some(entry) = entries
            .next_entry()
            .await
            .context("storage clear latest read dir")
            .as_internal_err()?
        {
            let metadata = entry
                .metadata()
                .await
                .context("storage clear latest read dir")
                .as_internal_err()?;
            if metadata.is_file() {
                let filename = entry.file_name().to_string_lossy().into_owned();
                files.push((filename, entry));
            }
        }

        // Sort files by filename (ascending order)
        files.sort_by(|a, b| a.0.cmp(&b.0));

        for (_filename, entry) in files.iter().take(files.len().saturating_sub(1)) {
            fs::remove_file(entry.path())
                .await
                .context("storage clear latest read dir")
                .as_internal_err()?;
        }

        Ok(())
    }

    fn paginate_rates_list<T>(rates: &[T], page: u32, size: u32) -> RatesList<T>
    where
        T: Clone,
    {
        let start = (page.saturating_sub(1) * size) as usize;
        let end = (start + size as usize).min(rates.len());

        let has_prev = start > 0;
        let rates_list = rates[start..end].to_vec();
        let has_next = end < rates.len(); // If there's more data beyond this page

        RatesList {
            has_prev,
            rates_list,
            has_next,
        }
    }
}

/// generate path to file from parent
fn generate_latest_file_path(date: DateTime<Utc>) -> String {
    let year = date.year();
    let month = date.month();
    let day = date.day();
    let hour = date.hour();
    let minute = date.minute();
    let second = date.second();

    let tostr = |n: u32| -> String {
        if n < 10 {
            return format!("0{n}");
        }
        n.to_string()
    };

    let filename = LATEST_FILENAME_FORMAT
        .replace("{YYYY}", year.to_string().as_str())
        .replace("{MM}", tostr(month).as_str())
        .replace("{DD}", tostr(day).as_str())
        .replace("{hh}", tostr(hour).as_str())
        .replace("{mm}", tostr(minute).as_str())
        .replace("{ss}", tostr(second).as_str());

    filename
}

/// generate path to file from parent
fn generate_historical_file_path(date: DateTime<Utc>) -> String {
    let year = date.year();
    let month = date.month();
    let day = date.day();

    let tostr = |n: u32| -> String {
        if n < 10 {
            format!("0{n}")
        } else {
            n.to_string()
        }
    };

    let filename = HISTORICAL_FILENAME_FORMAT
        .replace("{YYYY}", year.to_string().as_str())
        .replace("{MM}", tostr(month).as_str())
        .replace("{DD}", tostr(day).as_str());

    format!("{}/{}", year, filename)
}

fn parse_historical_file_path(filename: &str) -> Option<DateTime<Utc>> {
    if !filename.starts_with("historical-") || !filename.ends_with("Z.json") {
        return None;
    }

    // Extract only the date part (YYYY-MM-DD)
    let date_part = &filename["historical-".len()..filename.len() - "Z.json".len()];

    // Split into year, month, and day
    let mut parts = date_part.split('-');
    let year: i32 = parts.next()?.parse().ok()?;
    let month: u32 = parts.next()?.parse().ok()?;
    let day: u32 = parts.next()?.parse().ok()?;

    let date = Utc.with_ymd_and_hms(year, month, day, 0, 0, 0).single()?;
    Some(date)
}
#[cfg(test)]
mod forex_storage_impl_tests {
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn test_generate_latest_file_name() {
        let expected = "latest-2020-01-01T04:05:06Z.json";
        let date = Utc.with_ymd_and_hms(2020, 1, 1, 4, 5, 6).unwrap();
        let ret = generate_latest_file_path(date);
        println!("{ret}");
        assert_eq!(&ret, expected);

        let expected = "latest-2024-10-05T23:00:10Z.json";
        let date = Utc.with_ymd_and_hms(2024, 10, 5, 23, 0, 10).unwrap();
        let ret = generate_latest_file_path(date);
        println!("{ret}");
        assert_eq!(&ret, expected);
    }

    #[test]
    fn test_generate_historical_file_name() {
        let expected = "2020/historical-2020-01-01Z.json";
        let date = Utc.with_ymd_and_hms(2020, 1, 1, 4, 5, 6).unwrap();
        let ret = generate_historical_file_path(date);
        println!("{ret}");
        assert_eq!(&ret, expected);

        let expected = "2024/historical-2024-10-05Z.json";
        let date = Utc.with_ymd_and_hms(2024, 10, 5, 23, 0, 10).unwrap();
        let ret = generate_historical_file_path(date);
        println!("{ret}");
        assert_eq!(&ret, expected);
    }

    #[test]
    fn test_paginate_rates() {
        let v = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let expected = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let ret = ForexStorageImpl::paginate_rates_list(&v, 1, 8);
        dbg!(&ret);
        assert_eq!(ret.has_prev, false);
        assert_eq!(ret.has_next, true);
        assert_eq!(ret.rates_list, expected);
    }

    #[test]
    fn test_parse_historical_file_path() {
        let filename = "historical-2023-04-11Z.json";
        let expected = Utc.with_ymd_and_hms(2023, 4, 11, 0, 0, 0).unwrap();
        let ret = parse_historical_file_path(filename).unwrap();
        assert_eq!(ret, expected);
    }
}

#[async_trait]
impl ForexStorage for ForexStorageImpl {
    async fn insert_latest<T>(
        &self,
        date: DateTime<Utc>,
        rates: &RatesResponse<T>,
    ) -> ForexResult<()>
    where
        T: Debug + Serialize + for<'de> Deserialize<'de> + Send + Sync,
    {
        self.insert_latest(date, rates).await
    }

    async fn get_latest(&self) -> ForexResult<RatesResponse<Rates>> {
        self.get_latest().await
    }

    async fn insert_historical<T>(
        &self,
        date: DateTime<Utc>,
        rates: &RatesResponse<T>,
    ) -> ForexResult<()>
    where
        T: Debug + Serialize + for<'de> Deserialize<'de> + Send + Sync,
    {
        self.insert_historical(date, rates).await
    }

    async fn insert_historical_batch(
        &self,
        rates: Vec<RatesResponse<HistoricalRates>>,
    ) -> ForexResult<()> {
        self.insert_historical_batch(rates).await
    }

    async fn update_historical_rates_data(
        &self,
        date: DateTime<Utc>,
        new_data: Vec<Money>,
    ) -> ForexResult<RatesResponse<HistoricalRates>> {
        self.update_historical_rates_data(date, new_data).await
    }

    async fn get_historical(
        &self,
        date: DateTime<Utc>,
    ) -> ForexResult<RatesResponse<HistoricalRates>> {
        self.get_historical(date).await
    }

    async fn get_historical_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> ForexResult<Vec<RatesResponse<HistoricalRates>>> {
        self.get_historical_range(start, end).await
    }

    async fn get_latest_list(
        &self,
        page: u32,
        size: u32,
        order: Order,
    ) -> ForexResult<RatesList<RatesResponse<Rates>>> {
        self.get_latest_list(page, size, order).await
    }

    async fn get_historical_list(
        &self,
        page: u32,
        size: u32,
        order: Order,
    ) -> ForexResult<RatesList<RatesResponse<HistoricalRates>>> {
        self.get_historical_list(page, size, order).await
    }
}

#[async_trait]
impl ForexStorageDeletion for ForexStorageImpl {
    async fn clear_latest(&self) -> ForexResult<()> {
        self.clear_latest().await
    }
}

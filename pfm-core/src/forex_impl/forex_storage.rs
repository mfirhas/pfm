// forex_storage.rs implement storage mechanism for SERVER side http and cron.
// implementations for database to store forex data polled from the APIs.
// using filesystem with tokio

use std::fmt::Debug;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use crate::forex::entity::{HistoricalRates, Order, Rates, RatesList, RatesResponse};
use crate::forex::interface::{AsInternalError, ForexStorage};
use crate::forex::ForexError;
use crate::forex::ForexResult;
use crate::global::StorageFS;
use anyhow::Context;
use async_trait::async_trait;
use chrono::{DateTime, Datelike, Timelike, Utc};
use serde::{Deserialize, Serialize};
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;

const ERROR_PREFIX: &str = "[FOREX][storage_impl]";

const LATEST_FILENAME_FORMAT: &str = "latest-{YYYY}-{MM}-{DD}T{hh}:{mm}:{ss}Z.json";

const HISTORICAL_FILENAME_FORMAT: &str = "historical-{YYYY}-{MM}-{DD}Z.json";

const FILE_PERMISSION: u32 = 0o600;

#[derive(Clone)]
pub struct ForexStorageImpl {
    fs: StorageFS,
}

impl ForexStorageImpl {
    pub fn new(fs: StorageFS) -> Self {
        Self { fs }
    }

    async fn set_permission(pathbuf: &PathBuf) -> ForexResult<()> {
        // Set permissions to 600 (owner read/write only)
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

    async fn get_historical(
        &self,
        date: DateTime<Utc>,
    ) -> ForexResult<RatesResponse<HistoricalRates>> {
        self.get_historical(date).await
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

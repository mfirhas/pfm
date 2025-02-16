// implementations for database to store forex data polled from the APIs.
// using filesystem with tokio

use std::fmt::Debug;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use crate::forex::ForexError::{self, StorageError};
use crate::forex::{
    ForexResult, ForexStorage, ForexStorageRatesList, HistoricalRates, Order, Rates, RatesList,
    RatesResponse,
};
use crate::global::StorageFS;
use anyhow::anyhow;
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
            .map_err(|err| {
                StorageError(anyhow!(
                    "{} failed setting permission into file {:?}: {}",
                    ERROR_PREFIX,
                    &pathbuf.as_path(),
                    err
                ))
            })?
            .permissions();
        perms.set_mode(FILE_PERMISSION);
        fs::set_permissions(&pathbuf, perms).await.map_err(|err| {
            StorageError(anyhow!(
                "{} failed setting permission on {:?}: {}",
                ERROR_PREFIX,
                pathbuf.as_path(),
                err
            ))
        })?;

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
        let json_string = serde_json::to_string_pretty(&rates).map_err(|err| {
            StorageError(anyhow!(
                "{} failed parsing Rates into json string :{}",
                ERROR_PREFIX,
                err
            ))
        })?;

        let latest_write = self.fs.write().await;
        let latest_write = latest_write.latest();
        let latest_write = latest_write.join(generate_latest_file_name(date));

        let mut file = File::create(&latest_write).await.map_err(|err| {
            StorageError(anyhow!(
                "{} failed creating path {:?}: {}",
                ERROR_PREFIX,
                &latest_write.as_path(),
                err
            ))
        })?;
        file.write_all(json_string.as_bytes())
            .await
            .map_err(|err| {
                StorageError(anyhow!(
                    "{} failed writing into latest dir: {}",
                    ERROR_PREFIX,
                    err
                ))
            })?;
        file.flush().await.map_err(|err| {
            StorageError(anyhow!(
                "{} failed flushing insert_latest: {}",
                ERROR_PREFIX,
                err
            ))
        })?;

        Self::set_permission(&latest_write).await?;

        Ok(())
    }

    async fn get_latest(&self) -> ForexResult<RatesResponse<Rates>> {
        let latest_read = self.fs.read().await;
        let latest_read = latest_read.latest();

        let mut entries = fs::read_dir(latest_read).await.map_err(|err| {
            StorageError(anyhow!(
                "{} failed reading directory {:?} :{}",
                ERROR_PREFIX,
                &latest_read.as_path(),
                err
            ))
        })?;

        let mut files: Vec<PathBuf> = Vec::new();
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|err| StorageError(err.into()))?
        {
            let path = entry.path();
            files.push(path);
        }

        if files.is_empty() {
            return Err(StorageError(anyhow!(
                "{} latest directory is empty",
                ERROR_PREFIX
            )));
        }

        // sort descending
        files.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

        let content = fs::read_to_string(&files[0]).await.map_err(|err| {
            StorageError(anyhow!(
                "{} failed reading the content of file {:?}: {}",
                ERROR_PREFIX,
                &files[0].as_path(),
                err
            ))
        })?;

        let rates: RatesResponse<Rates> = serde_json::from_str(&content).map_err(|err| {
            StorageError(anyhow!(
                "{} failed parsing file content {:?} into Rates :{}",
                ERROR_PREFIX,
                &content,
                err
            ))
        })?;

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
        let json_string = serde_json::to_string_pretty(&rates).map_err(|err| {
            StorageError(anyhow!(
                "{} failed parsing Rates into json string :{}",
                ERROR_PREFIX,
                err
            ))
        })?;

        let historical_write = self.fs.write().await;
        let historical_write = historical_write.historical();
        let historical_write = historical_write.join(generate_historical_file_name(date));

        let mut file = File::create(&historical_write).await.map_err(|err| {
            StorageError(anyhow!(
                "{} failed creating path {:?}: {}",
                ERROR_PREFIX,
                &historical_write.as_path(),
                err
            ))
        })?;
        file.write_all(json_string.as_bytes())
            .await
            .map_err(|err| {
                StorageError(anyhow!(
                    "{} failed writing into historical dir: {}",
                    ERROR_PREFIX,
                    err
                ))
            })?;
        file.flush().await.map_err(|err| {
            StorageError(anyhow!(
                "{} failed flushing insert_latest: {}",
                ERROR_PREFIX,
                err
            ))
        })?;

        Self::set_permission(&historical_write).await?;

        Ok(())
    }

    async fn get_historical(
        &self,
        date: DateTime<Utc>,
    ) -> ForexResult<RatesResponse<HistoricalRates>> {
        let historical_read = self.fs.read().await;
        let historical_read = historical_read.latest();
        let filepath = historical_read.join(&generate_historical_file_name(date));

        let content = fs::read_to_string(&filepath).await.map_err(|err| {
            StorageError(anyhow!(
                "{} failed reading the content of file {:?}: {}",
                ERROR_PREFIX,
                &filepath.as_path(),
                err
            ))
        })?;

        let rates: RatesResponse<HistoricalRates> =
            serde_json::from_str(&content).map_err(|err| {
                StorageError(anyhow!(
                    "{} failed parsing file content {:?} into Rates :{}",
                    ERROR_PREFIX,
                    &content,
                    err
                ))
            })?;

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

        let mut entries = fs::read_dir(latest_read).await.map_err(|err| {
            StorageError(anyhow!(
                "{} failed reading directory {:?} :{}",
                ERROR_PREFIX,
                &latest_read.as_path(),
                err
            ))
        })?;

        let mut files: Vec<RatesResponse<Rates>> = Vec::new();
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|err| StorageError(err.into()))?
        {
            let path = entry.path();
            let content = tokio::fs::read_to_string(&path).await.map_err(|err| {
                ForexError::StorageError(anyhow!(
                    "{} failed getting latest list reading file content: {:?}: {}",
                    ERROR_PREFIX,
                    &path.as_path(),
                    err
                ))
            })?;
            let resp: RatesResponse<Rates> = serde_json::from_str(&content).map_err(|err| {
                ForexError::StorageError(anyhow!(
                    "{} failed getting latest list converting to RatesResponse<Rates>: {:?}: {}",
                    ERROR_PREFIX,
                    &path.as_path(),
                    err
                ))
            })?;
            files.push(resp);
        }

        if files.is_empty() {
            return Err(StorageError(anyhow!(
                "{} latest directory is empty",
                ERROR_PREFIX
            )));
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

        let mut entries = fs::read_dir(historical_read).await.map_err(|err| {
            StorageError(anyhow!(
                "{} failed reading directory {:?} :{}",
                ERROR_PREFIX,
                &historical_read.as_path(),
                err
            ))
        })?;

        let mut files: Vec<RatesResponse<HistoricalRates>> = Vec::new();
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|err| StorageError(err.into()))?
        {
            let path = entry.path();
            let content = tokio::fs::read_to_string(&path).await.map_err(|err| {
                ForexError::StorageError(anyhow!(
                    "{} failed getting latest list reading file content: {:?}: {}",
                    ERROR_PREFIX,
                    &path.as_path(),
                    err
                ))
            })?;
            let resp: RatesResponse<HistoricalRates> =
                serde_json::from_str(&content).map_err(|err| {
                    ForexError::StorageError(anyhow!(
                    "{} failed getting latest list converting to RatesResponse<Rates>: {:?}: {}",
                    ERROR_PREFIX,
                    &path.as_path(),
                    err
                ))
                })?;
            files.push(resp);
        }

        if files.is_empty() {
            return Err(StorageError(anyhow!(
                "{} latest directory is empty",
                ERROR_PREFIX
            )));
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

        let has_prev = start > 1;
        let rates_list = rates[start..end].to_vec();
        let has_next = end < rates.len(); // If there's more data beyond this page

        RatesList {
            has_prev,
            rates_list,
            has_next,
        }
    }
}

fn generate_latest_file_name(date: DateTime<Utc>) -> String {
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

fn generate_historical_file_name(date: DateTime<Utc>) -> String {
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

    filename
}

#[cfg(test)]
mod forex_storage_impl_tests {
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn test_generate_latest_file_name() {
        let expected = "latest-2020-01-01T04:05:06Z.json";
        let date = Utc.with_ymd_and_hms(2020, 1, 1, 4, 5, 6).unwrap();
        let ret = generate_latest_file_name(date);
        println!("{ret}");
        assert_eq!(&ret, expected);

        let expected = "latest-2024-10-05T23:00:10Z.json";
        let date = Utc.with_ymd_and_hms(2024, 10, 5, 23, 0, 10).unwrap();
        let ret = generate_latest_file_name(date);
        println!("{ret}");
        assert_eq!(&ret, expected);
    }

    #[test]
    fn test_generate_historical_file_name() {
        let expected = "historical-2020-01-01Z.json";
        let date = Utc.with_ymd_and_hms(2020, 1, 1, 4, 5, 6).unwrap();
        let ret = generate_historical_file_name(date);
        println!("{ret}");
        assert_eq!(&ret, expected);

        let expected = "historical-2024-10-05Z.json";
        let date = Utc.with_ymd_and_hms(2024, 10, 5, 23, 0, 10).unwrap();
        let ret = generate_historical_file_name(date);
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
}

#[async_trait]
impl ForexStorageRatesList for ForexStorageImpl {
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

// implementations for database to store forex data polled from the APIs.
// using filesystem with tokio

use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use crate::forex::{ForexResult, ForexStorage, HistoricalRates, Rates, RatesResponse};
use crate::global::StorageFS;
use anyhow::anyhow;
use async_trait::async_trait;
use chrono::{DateTime, Datelike, Timelike, Utc};
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;

const ERROR_PREFIX: &str = "[FOREX][storage_impl]";

const LATEST_FILENAME_FORMAT: &str = "latest-{YYYY}-{MM}-{DD}T{hh}:{mm}:{ss}Z.json";

const HISTORICAL_FILENAME_FORMAT: &str = "historical-{YYYY}-{MM}-{DD}Z.json";

const FILE_PERMISSION: u32 = 0o600;

pub(crate) struct ForexStorageImpl {
    fs: StorageFS,
}

impl ForexStorageImpl {
    pub(crate) fn new(fs: StorageFS) -> Self {
        Self { fs }
    }

    async fn set_permission(pathbuf: &PathBuf) -> ForexResult<()> {
        // Set permissions to 600 (owner read/write only)
        let mut perms = fs::metadata(&pathbuf)
            .await
            .map_err(|err| {
                anyhow!(
                    "{} failed setting permission into file {:?}: {}",
                    ERROR_PREFIX,
                    &pathbuf.as_path(),
                    err
                )
            })?
            .permissions();
        perms.set_mode(FILE_PERMISSION);
        fs::set_permissions(&pathbuf, perms).await?;

        Ok(())
    }

    async fn insert_latest(
        &self,
        date: DateTime<Utc>,
        rates: RatesResponse<Rates>,
    ) -> ForexResult<()> {
        let json_string = serde_json::to_string_pretty(&rates).map_err(|err| {
            anyhow!(
                "{} failed parsing Rates into json string :{}",
                ERROR_PREFIX,
                err
            )
        })?;

        let latest_write = self.fs.write().await;
        let latest_write = latest_write.latest();
        let latest_write = latest_write.join(generate_latest_file_name(date));

        let mut file = File::create(&latest_write).await?;
        file.write_all(json_string.as_bytes())
            .await
            .map_err(|err| anyhow!("{} failed writing into latest dir: {}", ERROR_PREFIX, err))?;
        file.flush().await?;

        Self::set_permission(&latest_write).await?;

        Ok(())
    }

    async fn get_latest(&self) -> ForexResult<RatesResponse<Rates>> {
        let latest_read = self.fs.read().await;
        let latest_read = latest_read.latest();

        let mut entries = fs::read_dir(latest_read).await.map_err(|err| {
            anyhow!(
                "{} failed reading directory {:?} :{}",
                ERROR_PREFIX,
                &latest_read.as_path(),
                err
            )
        })?;

        let mut files: Vec<PathBuf> = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            files.push(path);
        }

        if files.is_empty() {
            return Err(anyhow!("{} latest directory is empty", ERROR_PREFIX));
        }

        // sort descending
        files.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

        let content = fs::read_to_string(&files[0]).await.map_err(|err| {
            anyhow!(
                "{} failed reading the content of file {:?}: {}",
                ERROR_PREFIX,
                &files[0].as_path(),
                err
            )
        })?;

        let rates: RatesResponse<Rates> = serde_json::from_str(&content).map_err(|err| {
            anyhow!(
                "{} failed parsing file content {:?} into Rates :{}",
                ERROR_PREFIX,
                &content,
                err
            )
        })?;

        Ok(rates)
    }

    async fn insert_historical(
        &self,
        date: DateTime<Utc>,
        rates: RatesResponse<HistoricalRates>,
    ) -> ForexResult<()> {
        let json_string = serde_json::to_string_pretty(&rates).map_err(|err| {
            anyhow!(
                "{} failed parsing Rates into json string :{}",
                ERROR_PREFIX,
                err
            )
        })?;

        let historical_write = self.fs.write().await;
        let historical_write = historical_write.historical();
        let historical_write = historical_write.join(generate_historical_file_name(date));

        let mut file = File::create(&historical_write).await?;
        file.write_all(json_string.as_bytes())
            .await
            .map_err(|err| {
                anyhow!(
                    "{} failed writing into historical dir: {}",
                    ERROR_PREFIX,
                    err
                )
            })?;
        file.flush().await?;

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
            anyhow!(
                "{} failed reading the content of file {:?}: {}",
                ERROR_PREFIX,
                &filepath.as_path(),
                err
            )
        })?;

        let rates: RatesResponse<HistoricalRates> =
            serde_json::from_str(&content).map_err(|err| {
                anyhow!(
                    "{} failed parsing file content {:?} into Rates :{}",
                    ERROR_PREFIX,
                    &content,
                    err
                )
            })?;

        Ok(rates)
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
}

#[async_trait]
impl ForexStorage for ForexStorageImpl {
    async fn insert_latest(
        &self,
        date: DateTime<Utc>,
        rates: RatesResponse<Rates>,
    ) -> ForexResult<()> {
        self.insert_latest(date, rates).await
    }

    async fn get_latest(&self) -> ForexResult<RatesResponse<Rates>> {
        self.get_latest().await
    }

    async fn insert_historical(
        &self,
        date: DateTime<Utc>,
        rates: RatesResponse<HistoricalRates>,
    ) -> ForexResult<()> {
        self.insert_historical(date, rates).await
    }

    async fn get_historical(
        &self,
        date: DateTime<Utc>,
    ) -> ForexResult<RatesResponse<HistoricalRates>> {
        self.get_historical(date).await
    }
}

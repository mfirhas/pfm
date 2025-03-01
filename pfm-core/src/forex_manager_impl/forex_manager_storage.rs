// forex_manager_storage.rs implements storage mechanism for CLIENT side CLI or web

use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use crate::{
    forex::{Currency, Money},
    forex_manager::{
        Cash, CashListResponse, ForexManagerError::StorageError, ForexManagerResult,
        ForexManagerStorage, ForexPurchaseParams, Order,
    },
    global::ClientStorageFS,
};
use anyhow::anyhow;
use async_trait::async_trait;
use chrono::{DateTime, Datelike, Timelike, Utc};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
use uuid::Uuid;

const ERROR_PREFIX: &str = "[FOREX_MANAGER][storage_impl]";

const FILE_PERMISSION: u32 = 0o600;

const FOREX_FILENAME_FORMAT: &str = "{currency}-{YYYY}-{MM}-{DD}T{hh}:{mm}:{ss}Z.json";

#[derive(Clone)]
pub struct ForexManagerStorageImpl {
    fs: ClientStorageFS,
}

impl ForexManagerStorageImpl {
    async fn set_permission(pathbuf: &PathBuf) -> ForexManagerResult<()> {
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

    fn generate_forex_filename(currency: Currency, purchase_date: DateTime<Utc>) -> String {
        let curr = currency.to_string();
        let year = purchase_date.year();
        let month = purchase_date.month();
        let day = purchase_date.day();
        let hour = purchase_date.hour();
        let minute = purchase_date.minute();
        let second = purchase_date.second();

        let tostr = |n: u32| -> String {
            if n < 10 {
                return format!("0{n}");
            }
            n.to_string()
        };

        let filename = FOREX_FILENAME_FORMAT
            .replace("{currency}", &curr)
            .replace("{YYYY}", year.to_string().as_str())
            .replace("{MM}", tostr(month).as_str())
            .replace("{DD}", tostr(day).as_str())
            .replace("{hh}", tostr(hour).as_str())
            .replace("{mm}", tostr(minute).as_str())
            .replace("{ss}", tostr(second).as_str());

        filename
    }

    fn paginate_cash_list(cashes: &[Cash], page: u32, size: u32) -> CashListResponse {
        let start = (page.saturating_sub(1) * size) as usize;
        let end = (start + size as usize).min(cashes.len());

        let has_prev = start > 0;
        let cash_list: Vec<Cash> = cashes[start..end].to_vec();
        let has_next = end < cashes.len(); // If there's more data beyond this page

        CashListResponse {
            has_prev,
            cash_list,
            has_next,
        }
    }

    async fn insert(&self, entry: Cash) -> ForexManagerResult<()> {
        let json_string = serde_json::to_string_pretty(&entry).map_err(|err| {
            StorageError(anyhow!(
                "{} failed parsing Rates into json string :{}",
                ERROR_PREFIX,
                err
            ))
        })?;

        let currency = entry.money.currency();
        let date = entry.purchase_date;

        let forex_write = self.fs.write().await;
        let forex_write = forex_write.forex();
        let forex_write = forex_write.join(Self::generate_forex_filename(currency, date));

        let mut file = File::create(&forex_write).await.map_err(|err| {
            StorageError(anyhow!(
                "{} failed creating path {:?}: {}",
                ERROR_PREFIX,
                &forex_write.as_path(),
                err
            ))
        })?;
        file.write_all(json_string.as_bytes())
            .await
            .map_err(|err| {
                StorageError(anyhow!(
                    "{} failed writing into forex dir: {}",
                    ERROR_PREFIX,
                    err
                ))
            })?;
        file.flush().await.map_err(|err| {
            StorageError(anyhow!(
                "{} failed flushing insert forex: {}",
                ERROR_PREFIX,
                err
            ))
        })?;

        Self::set_permission(&forex_write).await?;

        Ok(())
    }

    /// get an entry from records
    async fn get(&self, id: Uuid) -> ForexManagerResult<Cash> {
        let forex_read = self.fs.read().await;
        let forex_read = forex_read.forex();

        let mut entries = fs::read_dir(forex_read).await.map_err(|err| {
            StorageError(anyhow!(
                "{} failed reading directory {:?} :{}",
                ERROR_PREFIX,
                &forex_read.as_path(),
                err
            ))
        })?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|err| StorageError(err.into()))?
        {
            let path = entry.path();
            let content = fs::read_to_string(&path).await.map_err(|err| {
                StorageError(anyhow!(
                    "{} failed reading the content of file {:?}: {}",
                    ERROR_PREFIX,
                    &path.as_path(),
                    err
                ))
            })?;

            let cash: Cash = serde_json::from_str(&content).map_err(|err| {
                StorageError(anyhow!(
                    "{} failed parsing file content {:?} into Cash :{}",
                    ERROR_PREFIX,
                    &content,
                    err
                ))
            })?;

            if cash.id == id {
                return Ok(cash);
            }
        }

        Err(StorageError(anyhow!(
            "{} forex entry not found",
            ERROR_PREFIX
        )))
    }

    /// get paginated list of entries
    async fn get_list(
        &self,
        page: u32,
        size: u32,
        order: Order,
    ) -> ForexManagerResult<CashListResponse> {
        let forex_read = self.fs.read().await;
        let forex_read = forex_read.forex();

        let mut entries = fs::read_dir(forex_read).await.map_err(|err| {
            StorageError(anyhow!(
                "{} failed reading directory {:?} :{}",
                ERROR_PREFIX,
                &forex_read.as_path(),
                err
            ))
        })?;

        let mut files: Vec<Cash> = Vec::new();
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|err| StorageError(err.into()))?
        {
            let path = entry.path();
            let content = tokio::fs::read_to_string(&path).await.map_err(|err| {
                StorageError(anyhow!(
                    "{} failed getting forex list reading file content: {:?}: {}",
                    ERROR_PREFIX,
                    &path.as_path(),
                    err
                ))
            })?;
            let resp: Cash = serde_json::from_str(&content).map_err(|err| {
                StorageError(anyhow!(
                    "{} failed getting forex list converting to Cash: {:?}: {}",
                    ERROR_PREFIX,
                    &path.as_path(),
                    err
                ))
            })?;
            files.push(resp);
        }

        if files.is_empty() {
            return Err(StorageError(anyhow!(
                "{} forex directory is empty",
                ERROR_PREFIX
            )));
        }

        match order {
            Order::ASC => files.sort_by_key(|cash| cash.purchase_date),
            Order::DESC => files.sort_by(|a, b| b.purchase_date.cmp(&a.purchase_date)),
        }

        let paginated = Self::paginate_cash_list(&files, page, size);

        let resp = CashListResponse {
            has_prev: paginated.has_prev,
            cash_list: paginated.cash_list,
            has_next: paginated.has_next,
        };

        Ok(resp)
    }

    /// edit existing forex records
    async fn update(&self, entry: Cash) -> ForexManagerResult<()> {
        let forex_write = self.fs.write().await;
        let forex_write = forex_write.forex();

        let mut entries = fs::read_dir(forex_write).await.map_err(|err| {
            StorageError(anyhow!(
                "{} failed reading directory {:?} :{}",
                ERROR_PREFIX,
                &forex_write.as_path(),
                err
            ))
        })?;

        while let Some(item) = entries
            .next_entry()
            .await
            .map_err(|err| StorageError(err.into()))?
        {
            let path = item.path();
            let content = fs::read_to_string(&path).await.map_err(|err| {
                StorageError(anyhow!(
                    "{} failed reading the content of file {:?}: {}",
                    ERROR_PREFIX,
                    &path.as_path(),
                    err
                ))
            })?;

            let cash: Cash = serde_json::from_str(&content).map_err(|err| {
                StorageError(anyhow!(
                    "{} failed parsing file content {:?} into Cash :{}",
                    ERROR_PREFIX,
                    &content,
                    err
                ))
            })?;

            if cash.id == entry.id {
                let json_string = serde_json::to_string_pretty(&entry).map_err(|err| {
                    StorageError(anyhow!(
                        "{} failed parsing Rates into json string :{}",
                        ERROR_PREFIX,
                        err
                    ))
                })?;

                fs::write(&path, json_string.as_bytes())
                    .await
                    .map_err(|err| {
                        StorageError(anyhow!(
                            "{} failed to overwrite content of path {:?}: {}",
                            ERROR_PREFIX,
                            &path.as_path(),
                            err
                        ))
                    })?;

                return Ok(());
            }
        }

        Err(StorageError(anyhow!(
            "{} forex entry to update not found",
            ERROR_PREFIX
        )))
    }

    /// remove an entry from existing records
    async fn delete(&self, id: Uuid) -> ForexManagerResult<()> {
        let forex_write = self.fs.write().await;
        let forex_write = forex_write.forex();

        let mut entries = fs::read_dir(forex_write).await.map_err(|err| {
            StorageError(anyhow!(
                "{} failed reading directory {:?} :{}",
                ERROR_PREFIX,
                &forex_write.as_path(),
                err
            ))
        })?;

        while let Some(item) = entries
            .next_entry()
            .await
            .map_err(|err| StorageError(err.into()))?
        {
            let path = item.path();
            let content = fs::read_to_string(&path).await.map_err(|err| {
                StorageError(anyhow!(
                    "{} failed reading the content of file {:?}: {}",
                    ERROR_PREFIX,
                    &path.as_path(),
                    err
                ))
            })?;

            let cash: Cash = serde_json::from_str(&content).map_err(|err| {
                StorageError(anyhow!(
                    "{} failed parsing file content {:?} into Cash :{}",
                    ERROR_PREFIX,
                    &content,
                    err
                ))
            })?;

            if cash.id == id {
                fs::remove_file(&path).await.map_err(|err| {
                    StorageError(anyhow!(
                        "{} failed deleting file {:?}: {}",
                        ERROR_PREFIX,
                        &path.as_path(),
                        err
                    ))
                })?;

                return Ok(());
            }
        }

        Err(StorageError(anyhow!(
            "{} forex entry to update not found",
            ERROR_PREFIX
        )))
    }
}

#[cfg(test)]
mod forex_manager_storage_tests {
    use chrono::{TimeZone, Utc};

    use crate::forex::{Currency, Money};

    use super::ForexManagerStorageImpl;

    #[test]
    fn test_generate_forex_filename() {
        let expected = "USD-2025-01-01T10:10:10Z.json";
        let input_currency = Currency::USD;
        let purchase_date = Utc.with_ymd_and_hms(2025, 1, 1, 10, 10, 10).unwrap();
        let ret = ForexManagerStorageImpl::generate_forex_filename(input_currency, purchase_date);

        dbg!(&ret);

        assert_eq!(expected, ret.as_str());
    }
}

#[async_trait]
impl ForexManagerStorage for ForexManagerStorageImpl {
    async fn insert(&self, entry: Cash) -> ForexManagerResult<()> {
        self.insert(entry).await
    }

    /// get an entry from records
    async fn get(&self, id: Uuid) -> ForexManagerResult<Cash> {
        self.get(id).await
    }

    /// get paginated list of entries
    async fn get_list(
        &self,
        page: u32,
        size: u32,
        order: Order,
    ) -> ForexManagerResult<CashListResponse> {
        self.get_list(page, size, order).await
    }

    /// edit existing forex records
    async fn update(&self, entry: Cash) -> ForexManagerResult<()> {
        self.update(entry).await
    }

    /// remove an entry from existing records
    async fn delete(&self, id: Uuid) -> ForexManagerResult<()> {
        self.delete(id).await
    }
}

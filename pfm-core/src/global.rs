// global.rs contains global variables

use anyhow::anyhow;
use anyhow::Result;
use lazy_static::lazy_static;
use reqwest::Client;
use serde::Deserialize;
use std::{fmt::Debug, path::PathBuf, sync::Arc, time::Duration};
use tokio::sync::RwLock;

use crate::forex::Currency;
use crate::utils;

const ENV_PREFIX: &str = "CORE_";
const ERROR_PREFIX: &str = "[GLOBAL]";

///////////////////////////////////// STORAGE FILESYSTEM FOR SERVER /////////////////////////////////////
// 7 = read (4) + write (2) + execute (1) for owner
// 0 = no permissions for group
// 0 = no permissions for others
const STORAGE_FS_PERMISSION: u32 = 0o750;
///////////////////////////////////// STORAGE FILESYSTEM FOR SERVER (END) /////////////////////////////////////

///////////////////////////////////// STORAGE FILESYSTEM FOR CLIENT(CLI) /////////////////////////////////////
// 7 = read (4) + write (2) + execute (1) for owner
// 0 = no permissions for group
// 0 = no permissions for others
const CLIENT_STORAGE_FS_PERMISSION: u32 = 0o700;

/// path to storage for client-side data
const CLIENT_STORAGE_FS_DIR_PATH: &str = "TODO";

/// path to storage for client-side data for development, placed in workspace root
const CLIENT_STORAGE_FS_DIR_PATH_DEV: &str = "test_dir_client";
///////////////////////////////////// STORAGE FILESYSTEM FOR CLIENT(CLI) (END) /////////////////////////////////////

/// Get instantiated global http client object.
pub fn http_client() -> Client {
    HTTP_CLIENT.clone()
}

/// Get instantiated global config object.
pub fn config() -> &'static Config {
    &CONFIG
}

/// Get instantiated global storage filesystem object for SERVER.
pub fn storage_fs() -> StorageFS {
    STORAGE_FS.clone()
}

pub fn client_storage_fs() -> ClientStorageFS {
    CLIENT_STORAGE_FS.clone()
}

pub const BASE_CURRENCY: Currency = Currency::USD;

lazy_static! {
    static ref CONFIG: Config = init_config().expect("failed init core config");
    static ref HTTP_CLIENT: Client = init_http_client().expect("failed init core http client");
    static ref STORAGE_FS: StorageFS = init_storage_fs().expect("failed init storage fs");
    static ref CLIENT_STORAGE_FS: ClientStorageFS =
        init_client_storage_fs().expect("failed init client storage fs");
    /// Directory for server-side storage.
    /// For local development, using project's workspace root in test_dir/
    static ref STORAGE_DIR_PATH: PathBuf = {
        if cfg!(debug_assertions) {
            let local_dev_path = "test_dir";
            let workspace_dir = utils::find_workspace_root().expect("init storage dir path error");
            let path = workspace_dir.join(local_dev_path);
            return path;
        }

        #[cfg(target_os = "windows")]
        {
            panic!("Sorry, development and server on Windows not supported at the moment.");
        }

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            let default_location = format!("{}/pfm", dirs::home_dir().expect("failed initializing production pfm data path").to_string_lossy().to_string());
            // APP_DATA_PATH is set from env var in prod to determine where pfm data to be stored.
            // set APP_DATA_PATH to path to pfm, e.g. /home/myuser/pfm, or /Users/myuser/pfm
            let location = std::env::var("APP_DATA_PATH").unwrap_or(default_location);
            let dir_name = "pfm-data";
            let storage_dir_path = PathBuf::from(location).join(dir_name);
            return storage_dir_path;
        }
    };
}

fn init_http_client() -> Result<reqwest::Client, anyhow::Error> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .pool_idle_timeout(Duration::from_secs(300))
        .pool_max_idle_per_host(32)
        .build()
        .map_err(|err| anyhow::anyhow!("{} failed creating http client: {}", ERROR_PREFIX, err));

    client
}

fn init_config<CFG>() -> Result<CFG, anyhow::Error>
where
    CFG: for<'de> Deserialize<'de> + Debug + Clone,
{
    let cfg = utils::get_config(ENV_PREFIX);

    cfg
}

/// Alias for ServerFS, Filesystem for storing data at server side.
pub type StorageFS = Arc<RwLock<ServerFS>>;

#[derive(Debug)]
pub struct ServerFS {
    root: PathBuf,
    latest: PathBuf,
    historical: PathBuf,
}

impl ServerFS {
    pub(crate) fn is_dir(&self) -> bool {
        self.root.is_dir() && self.latest.is_dir() && self.historical.is_dir()
    }

    pub(crate) fn root(&self) -> &PathBuf {
        &self.root
    }

    pub(crate) fn latest(&self) -> &PathBuf {
        &self.latest
    }

    pub(crate) fn historical(&self) -> &PathBuf {
        &self.historical
    }
}

fn init_storage_fs() -> Result<StorageFS, anyhow::Error> {
    let root_pb = STORAGE_DIR_PATH.clone();

    let root = utils::set_root(root_pb, STORAGE_FS_PERMISSION).map_err(|err| {
        anyhow!(
            "{} failed initializing server storage fs: {}",
            ERROR_PREFIX,
            err
        )
    })?;

    let latest = utils::set_sub_dir(&root, "latest", STORAGE_FS_PERMISSION).map_err(|err| {
        anyhow!(
            "{} failed initializing server storage fs latest dir: {}",
            ERROR_PREFIX,
            err
        )
    })?;

    let historical =
        utils::set_sub_dir(&root, "historical", STORAGE_FS_PERMISSION).map_err(|err| {
            anyhow!(
                "{} failed initializing server storage fs historical dir: {}",
                ERROR_PREFIX,
                err
            )
        })?;

    // initiate historical
    let storage_fs = Arc::new(RwLock::new(ServerFS {
        root,
        latest,
        historical,
    }));

    Ok(storage_fs)
}

pub type ClientStorageFS = Arc<RwLock<ClientFS>>;

#[derive(Debug)]
pub struct ClientFS {
    root: PathBuf,
    forex: PathBuf,
    pm: PathBuf,
}

impl ClientFS {
    pub(crate) fn is_dir(&self) -> bool {
        self.root.is_dir() && self.forex.is_dir() && self.pm.is_dir()
    }

    pub(crate) fn root(&self) -> &PathBuf {
        &self.root
    }

    pub(crate) fn forex(&self) -> &PathBuf {
        &self.forex
    }

    pub(crate) fn pm(&self) -> &PathBuf {
        &self.pm
    }
}

fn init_client_storage_fs() -> Result<ClientStorageFS, anyhow::Error> {
    let root_pb = if cfg!(debug_assertions) {
        let workspace_dir = utils::find_workspace_root()?;
        let path = workspace_dir.join(CLIENT_STORAGE_FS_DIR_PATH_DEV);
        path
    } else {
        PathBuf::from(CLIENT_STORAGE_FS_DIR_PATH)
    };

    let root = utils::set_root(root_pb, CLIENT_STORAGE_FS_PERMISSION).map_err(|err| {
        anyhow!(
            "{} failed initializing client storage fs: {}",
            ERROR_PREFIX,
            err
        )
    })?;

    let forex =
        utils::set_sub_dir(&root, "forex", CLIENT_STORAGE_FS_PERMISSION).map_err(|err| {
            anyhow!(
                "{} failed initializing client storage fs forex dir: {}",
                ERROR_PREFIX,
                err
            )
        })?;

    let pm = utils::set_sub_dir(&root, "pm", CLIENT_STORAGE_FS_PERMISSION).map_err(|err| {
        anyhow!(
            "{} failed initializing client storage fs pm dir: {}",
            ERROR_PREFIX,
            err
        )
    })?;

    // initiate historical
    let client_storage_fs = Arc::new(RwLock::new(ClientFS { root, forex, pm }));

    Ok(client_storage_fs)
}

/// Configurations
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Using symbol or code for displaying money.
    #[serde(alias = "CORE_FOREX_USE_SYMBOL", default)]
    pub forex_use_symbol: bool,

    /// API key for https://currencyapi.com
    #[serde(alias = "CORE_FOREX_CURRENCY_API_KEY")]
    pub forex_currency_api_key: String,

    /// API key for https://openexchangerates.org
    #[serde(alias = "CORE_FOREX_OPEN_EXCHANGE_API_KEY")]
    pub forex_open_exchange_api_key: String,

    #[serde(alias = "CORE_FOREX_CURRENCYBEACON_API_KEY")]
    pub forex_currencybeacon_api_key: String,

    #[serde(alias = "CORE_FOREX_TRADERMADE_API_KEY")]
    pub forex_tradermade_api_key: String,
}

#[cfg(test)]
mod global_tests {
    use super::*;
    use std::{fs, os::unix::fs::PermissionsExt};

    #[test]
    fn test_config() {
        let cfg = init_config::<Config>();
        dbg!(&cfg);
        assert!(cfg.is_ok());
    }

    #[tokio::test]
    async fn test_dev_storage_fs_path() {
        let ret = init_storage_fs();

        dbg!(&ret);

        let storage_fs = ret.unwrap();
        let storage_fs = storage_fs.read().await;
        let is_dir = storage_fs.is_dir();
        assert!(is_dir);

        let storage_fs = &*storage_fs;
        let storage_fs_root = &*storage_fs.root;
        let storage_fs_latest = &storage_fs.latest;
        let storage_fs_historical = &storage_fs.historical;

        let metadata_root = fs::metadata(storage_fs_root).unwrap();
        let metadata_latest = fs::metadata(storage_fs_latest).unwrap();
        let metadata_historical = fs::metadata(storage_fs_historical).unwrap();

        assert_eq!(
            metadata_root.permissions().mode() & STORAGE_FS_PERMISSION,
            STORAGE_FS_PERMISSION
        );

        assert_eq!(
            metadata_latest.permissions().mode() & STORAGE_FS_PERMISSION,
            STORAGE_FS_PERMISSION
        );
        assert_eq!(
            metadata_historical.permissions().mode() & STORAGE_FS_PERMISSION,
            STORAGE_FS_PERMISSION
        );
    }
}

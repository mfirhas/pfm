// global.rs contains global variables

use anyhow::anyhow;
use anyhow::Result;
use configrs::config::Config as config_rs;
use lazy_static::lazy_static;
use reqwest::Client;
use serde::Deserialize;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::{
    fmt::Debug,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    time::Duration,
};

const ENV_PREFIX: &str = "CORE_";
const ERROR_PREFIX: &str = "[GLOBAL]";

///////////////////////////////////// STORAGE FILESYSTEM FOR SERVER /////////////////////////////////////
const STORAGE_FS_PERMISSION: u32 = 0o700;

/// storage filesystem for production in server.
const STORAGE_FS_PATH: &str = "TODO";

/// storage filesystem for local development, inside project directory.
const STORAGE_FS_PATH_DEV: &str = "./test_dir/";
///////////////////////////////////// STORAGE FILESYSTEM FOR SERVER (END) /////////////////////////////////////

/// path to .env file for development
pub(super) const DEV_ENV_PATH: &str = "./src/core.env";

/// Get instantiated global http client object.
pub(crate) fn http_client() -> &'static Client {
    &HTTP_CLIENT
}

/// Get instantiated global config object.
pub(crate) fn config() -> &'static Config {
    &CONFIG
}

/// Get instantiated global storage filesystem object for SERVER.
pub(crate) fn storage_fs() -> StorageFS {
    STORAGE_FS.clone()
}

lazy_static! {
    static ref CONFIG: Config = init_config().expect("failed init core config");
    static ref HTTP_CLIENT: Client = init_http_client().expect("failed init core http client");
    static ref STORAGE_FS: StorageFS = init_storage_fs().expect("failed init storage fs");
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
    let cfg = config_rs::new().with_env_prefix(ENV_PREFIX);

    // for local development config from file
    if cfg!(debug_assertions) {
        let cfg = cfg.with_env(DEV_ENV_PATH).build::<CFG>().map_err(|err| {
            anyhow::anyhow!(
                "{} failed parsing dev config from file {}: {}",
                ERROR_PREFIX,
                ENV_PREFIX,
                err
            )
        });

        return cfg;
    }

    let cfg = cfg
        .build::<CFG>()
        .map_err(|err| anyhow::anyhow!("{} failed parsing env config: {}", ERROR_PREFIX, err));

    cfg
}

pub type StorageFS = Arc<RwLock<PathBuf>>;

fn init_storage_fs() -> Result<StorageFS, anyhow::Error> {
    let path = if cfg!(debug_assertions) {
        Path::new(STORAGE_FS_PATH_DEV)
    } else {
        Path::new(STORAGE_FS_PATH)
    };

    let pb = path.to_path_buf();

    let is_exist = pb.try_exists().map_err(|err| {
        anyhow!(
            "{} failed checking root storage directory: {}",
            ERROR_PREFIX,
            err
        )
    })?;

    if !is_exist {
        // create the root dir
        fs::create_dir_all(&pb).map_err(|err| {
            anyhow!(
                "{} failed creating root directory at {:?} for storage fs: {}",
                ERROR_PREFIX,
                &pb.as_path(),
                err
            )
        })?;
    }

    // set permissions
    let metadata = fs::metadata(&pb).map_err(|err| {
        anyhow!(
            "{} failed to read metadata of {:?}: {}",
            ERROR_PREFIX,
            &pb.as_path(),
            err
        )
    })?;
    let mut new_permissions = metadata.permissions();
    // 7 = read (4) + write (2) + execute (1) for owner
    // 0 = no permissions for group
    // 0 = no permissions for others
    new_permissions.set_mode(STORAGE_FS_PERMISSION);
    fs::set_permissions(&pb, new_permissions).map_err(|err| {
        anyhow!(
            "{} failed setting permission into {:?}: {}",
            ERROR_PREFIX,
            &pb.as_path(),
            err
        )
    })?;

    let storage_fs = Arc::new(RwLock::new(pb));

    Ok(storage_fs)
}

/// Configurations
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Config {
    /// Using symbol or code for displaying money.
    #[serde(alias = "CORE_FOREX_USE_SYMBOL", default)]
    pub forex_use_symbol: bool,

    /// API key for https://currencyapi.com
    #[serde(alias = "CORE_FOREX_CURRENCY_API_KEY")]
    pub forex_currency_api_key: String,

    /// API key for https://openexchangerates.org
    #[serde(alias = "CORE_FOREX_OPEN_EXCHANGE_API_KEY")]
    pub forex_open_exchange_api_key: String,
}

#[cfg(test)]
mod global_tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_dev_env_path() {
        let path = Path::new(DEV_ENV_PATH);

        assert!(path.exists());
    }

    #[test]
    fn test_config() {
        let cfg = init_config::<Config>();
        dbg!(&cfg);
        assert!(cfg.is_ok());
    }

    #[test]
    fn test_dev_storage_fs_path() {
        let ret = init_storage_fs();

        dbg!(&ret);

        assert!(ret.is_ok());

        let ret = ret.unwrap();
        let is_dir = ret.read().unwrap().is_dir();

        assert!(is_dir);

        let pb = &*ret.read().unwrap();
        let metadata = fs::metadata(pb);

        assert!(metadata.is_ok());

        let md = metadata.unwrap();

        assert_eq!(
            md.permissions().mode() & STORAGE_FS_PERMISSION,
            STORAGE_FS_PERMISSION
        );
    }
}

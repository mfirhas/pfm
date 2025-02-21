// global.rs contains global variables

use anyhow::anyhow;
use anyhow::Result;
use configrs::config::Config as config_rs;
use lazy_static::lazy_static;
use reqwest::Client;
use serde::Deserialize;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::{fmt::Debug, path::PathBuf, sync::Arc, time::Duration};
use tokio::sync::RwLock;

use crate::forex::Currencies;
use crate::utils;

const ENV_PREFIX: &str = "CORE_";
const ERROR_PREFIX: &str = "[GLOBAL]";

///////////////////////////////////// STORAGE FILESYSTEM FOR SERVER /////////////////////////////////////
const STORAGE_FS_PERMISSION: u32 = 0o700;

/// path to storage for server-side data.
/// filled with absolute path to production storage location.
const STORAGE_FS_DIR_PATH: &str = "TODO";

/// directory name for local development for server-side data, to be placed in workspace root
const STORAGE_FS_DIR_PATH_DEV: &str = "test_dir";
///////////////////////////////////// STORAGE FILESYSTEM FOR SERVER (END) /////////////////////////////////////

///////////////////////////////////// STORAGE FILESYSTEM FOR CLIENT(CLI) /////////////////////////////////////
const CLIENT_STORAGE_FS_PERMISSION: u32 = 0o700;

/// path to storage for client-side data
const CLIENT_STORAGE_FS_DIR_PATH: &str = "TODO";

/// path to storage for client-side data for development, placed in workspace root
const CLIENT_STORAGE_FS_DIR_PATH_DEV: &str = "test_dir_client";
///////////////////////////////////// STORAGE FILESYSTEM FOR CLIENT(CLI) (END) /////////////////////////////////////

/// .env file for local development, to be placed in workspace root
/// create this file in workspace root.
const DEV_ENV_FILE_PATH: &str = ".env";

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

pub const BASE_CURRENCY: Currencies = Currencies::USD;
lazy_static! {
    static ref CONFIG: Config = init_config().expect("failed init core config");
    static ref HTTP_CLIENT: Client = init_http_client().expect("failed init core http client");
    static ref STORAGE_FS: StorageFS = init_storage_fs().expect("failed init storage fs");
    static ref CLIENT_STORAGE_FS: ClientStorageFS =
        init_client_storage_fs().expect("failed init client storage fs");
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
        let workspace_root = utils::find_workspace_root()?;
        let dev_env_path = workspace_root.join(DEV_ENV_FILE_PATH);
        let cfg = cfg.with_env(dev_env_path).build::<CFG>().map_err(|err| {
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
    let path = if cfg!(debug_assertions) {
        let workspace_dir = utils::find_workspace_root()?;
        let path = workspace_dir.join(STORAGE_FS_DIR_PATH_DEV);
        path
    } else {
        PathBuf::from(STORAGE_FS_DIR_PATH)
    };

    // initiate Root
    let root = path.to_path_buf();

    let is_exist = root.try_exists().map_err(|err| {
        anyhow!(
            "{} failed checking root storage directory: {}",
            ERROR_PREFIX,
            err
        )
    })?;

    if !is_exist {
        // create the root dir
        fs::create_dir_all(&root).map_err(|err| {
            anyhow!(
                "{} failed creating root directory at {:?} for storage fs: {}",
                ERROR_PREFIX,
                &root.as_path(),
                err
            )
        })?;
    }

    // set permissions
    let metadata = fs::metadata(&root).map_err(|err| {
        anyhow!(
            "{} failed to read metadata of {:?}: {}",
            ERROR_PREFIX,
            &root.as_path(),
            err
        )
    })?;
    let mut new_permissions = metadata.permissions();
    // 7 = read (4) + write (2) + execute (1) for owner
    // 0 = no permissions for group
    // 0 = no permissions for others
    new_permissions.set_mode(STORAGE_FS_PERMISSION);
    fs::set_permissions(&root, new_permissions).map_err(|err| {
        anyhow!(
            "{} failed setting permission into {:?}: {}",
            ERROR_PREFIX,
            &root.as_path(),
            err
        )
    })?;

    let latest = init_storage_fs_latest(&root)
        .map_err(|err| anyhow!("{} failed initiating latest dir: {}", ERROR_PREFIX, err))?;

    let historical = init_storage_fs_historical(&root)
        .map_err(|err| anyhow!("{} failed initiating historical dir: {}", ERROR_PREFIX, err))?;

    // initiate historical
    let storage_fs = Arc::new(RwLock::new(ServerFS {
        root,
        latest,
        historical,
    }));

    Ok(storage_fs)
}

fn init_storage_fs_latest(root: &PathBuf) -> Result<PathBuf, anyhow::Error> {
    // initiate latest
    let latest = root.join("latest");

    let is_exist = latest.try_exists().map_err(|err| {
        anyhow!(
            "{} failed checking latest storage directory: {}",
            ERROR_PREFIX,
            err
        )
    })?;

    if !is_exist {
        // create the latest dir
        fs::create_dir_all(&latest).map_err(|err| {
            anyhow!(
                "{} failed creating latest directory at {:?} for storage fs: {}",
                ERROR_PREFIX,
                &latest.as_path(),
                err
            )
        })?;
    }

    // set permissions
    let metadata = fs::metadata(&latest).map_err(|err| {
        anyhow!(
            "{} failed to read metadata of {:?}: {}",
            ERROR_PREFIX,
            &latest.as_path(),
            err
        )
    })?;
    let mut new_permissions = metadata.permissions();
    // 7 = read (4) + write (2) + execute (1) for owner
    // 0 = no permissions for group
    // 0 = no permissions for others
    new_permissions.set_mode(STORAGE_FS_PERMISSION);
    fs::set_permissions(&latest, new_permissions).map_err(|err| {
        anyhow!(
            "{} failed setting permission into {:?}: {}",
            ERROR_PREFIX,
            &latest.as_path(),
            err
        )
    })?;

    Ok(latest)
}

fn init_storage_fs_historical(root: &PathBuf) -> Result<PathBuf, anyhow::Error> {
    // initiate historical
    let historical = root.join("historical");

    let is_exist = historical.try_exists().map_err(|err| {
        anyhow!(
            "{} failed checking historical storage directory: {}",
            ERROR_PREFIX,
            err
        )
    })?;

    if !is_exist {
        // create the historical dir
        fs::create_dir_all(&historical).map_err(|err| {
            anyhow!(
                "{} failed creating historical directory at {:?} for storage fs: {}",
                ERROR_PREFIX,
                &historical.as_path(),
                err
            )
        })?;
    }

    // set permissions
    let metadata = fs::metadata(&historical).map_err(|err| {
        anyhow!(
            "{} failed to read metadata of {:?}: {}",
            ERROR_PREFIX,
            &historical.as_path(),
            err
        )
    })?;
    let mut new_permissions = metadata.permissions();
    // 7 = read (4) + write (2) + execute (1) for owner
    // 0 = no permissions for group
    // 0 = no permissions for others
    new_permissions.set_mode(STORAGE_FS_PERMISSION);
    fs::set_permissions(&historical, new_permissions).map_err(|err| {
        anyhow!(
            "{} failed setting permission into {:?}: {}",
            ERROR_PREFIX,
            &historical.as_path(),
            err
        )
    })?;

    Ok(historical)
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
    let path = if cfg!(debug_assertions) {
        let workspace_dir = utils::find_workspace_root()?;
        let path = workspace_dir.join(CLIENT_STORAGE_FS_DIR_PATH_DEV);
        path
    } else {
        PathBuf::from(CLIENT_STORAGE_FS_DIR_PATH)
    };

    // initiate Root
    let root = path.to_path_buf();

    let is_exist = root.try_exists().map_err(|err| {
        anyhow!(
            "{} failed checking root storage directory: {}",
            ERROR_PREFIX,
            err
        )
    })?;

    if !is_exist {
        // create the root dir
        fs::create_dir_all(&root).map_err(|err| {
            anyhow!(
                "{} failed creating root directory at {:?} for storage fs: {}",
                ERROR_PREFIX,
                &root.as_path(),
                err
            )
        })?;
    }

    // set permissions
    let metadata = fs::metadata(&root).map_err(|err| {
        anyhow!(
            "{} failed to read metadata of {:?}: {}",
            ERROR_PREFIX,
            &root.as_path(),
            err
        )
    })?;
    let mut new_permissions = metadata.permissions();
    // 7 = read (4) + write (2) + execute (1) for owner
    // 0 = no permissions for group
    // 0 = no permissions for others
    new_permissions.set_mode(CLIENT_STORAGE_FS_PERMISSION);
    fs::set_permissions(&root, new_permissions).map_err(|err| {
        anyhow!(
            "{} failed setting permission into {:?}: {}",
            ERROR_PREFIX,
            &root.as_path(),
            err
        )
    })?;

    let forex = init_client_storage_fs_forex(&root)
        .map_err(|err| anyhow!("{} failed initiating forex dir: {}", ERROR_PREFIX, err))?;

    let pm = init_client_storage_fs_pm(&root)
        .map_err(|err| anyhow!("{} failed initiating pm dir: {}", ERROR_PREFIX, err))?;

    // initiate historical
    let client_storage_fs = Arc::new(RwLock::new(ClientFS { root, forex, pm }));

    Ok(client_storage_fs)
}

fn init_client_storage_fs_forex(root: &PathBuf) -> Result<PathBuf, anyhow::Error> {
    // initiate forex
    let forex = root.join("forex");

    let is_exist = forex.try_exists().map_err(|err| {
        anyhow!(
            "{} failed checking forex storage directory: {}",
            ERROR_PREFIX,
            err
        )
    })?;

    if !is_exist {
        // create the forex dir
        fs::create_dir_all(&forex).map_err(|err| {
            anyhow!(
                "{} failed creating forex directory at {:?} for storage fs: {}",
                ERROR_PREFIX,
                &forex.as_path(),
                err
            )
        })?;
    }

    // set permissions
    let metadata = fs::metadata(&forex).map_err(|err| {
        anyhow!(
            "{} failed to read metadata of {:?}: {}",
            ERROR_PREFIX,
            &forex.as_path(),
            err
        )
    })?;
    let mut new_permissions = metadata.permissions();
    // 7 = read (4) + write (2) + execute (1) for owner
    // 0 = no permissions for group
    // 0 = no permissions for others
    new_permissions.set_mode(CLIENT_STORAGE_FS_PERMISSION);
    fs::set_permissions(&forex, new_permissions).map_err(|err| {
        anyhow!(
            "{} failed setting permission into {:?}: {}",
            ERROR_PREFIX,
            &forex.as_path(),
            err
        )
    })?;

    Ok(forex)
}

fn init_client_storage_fs_pm(root: &PathBuf) -> Result<PathBuf, anyhow::Error> {
    // initiate pm
    let pm = root.join("pm");

    let is_exist = pm.try_exists().map_err(|err| {
        anyhow!(
            "{} failed checking pm storage directory: {}",
            ERROR_PREFIX,
            err
        )
    })?;

    if !is_exist {
        // create the pm dir
        fs::create_dir_all(&pm).map_err(|err| {
            anyhow!(
                "{} failed creating pm directory at {:?} for storage fs: {}",
                ERROR_PREFIX,
                &pm.as_path(),
                err
            )
        })?;
    }

    // set permissions
    let metadata = fs::metadata(&pm).map_err(|err| {
        anyhow!(
            "{} failed to read metadata of {:?}: {}",
            ERROR_PREFIX,
            &pm.as_path(),
            err
        )
    })?;
    let mut new_permissions = metadata.permissions();
    // 7 = read (4) + write (2) + execute (1) for owner
    // 0 = no permissions for group
    // 0 = no permissions for others
    new_permissions.set_mode(CLIENT_STORAGE_FS_PERMISSION);
    fs::set_permissions(&pm, new_permissions).map_err(|err| {
        anyhow!(
            "{} failed setting permission into {:?}: {}",
            ERROR_PREFIX,
            &pm.as_path(),
            err
        )
    })?;

    Ok(pm)
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
}

#[cfg(test)]
mod global_tests {
    use super::*;

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

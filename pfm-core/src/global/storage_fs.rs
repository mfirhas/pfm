use anyhow::Context;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::LazyLock;
use tokio::sync::RwLock;

use crate::utils;

/// Get instantiated global storage filesystem object for SERVER.
pub fn storage_fs() -> StorageFS {
    STORAGE_FS.clone()
}

static STORAGE_FS: LazyLock<StorageFS> =
    LazyLock::new(|| init_storage_fs().expect("global init storage fs"));

const STORAGE_FS_PERMISSION: u32 = 0o750;
const STORAGE_FS_LATEST_DIR_NAME: &str = "latest";
const STORAGE_FS_HISTORICAL_DIR_NAME: &str = "historical";

/// Directory for server-side storage.
/// For local development, using project's workspace root in test_dir/
static STORAGE_FS_DIR_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
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
        let default_location = format!(
            "{}/pfm",
            dirs::home_dir()
                .expect("failed initializing production pfm data path")
                .to_string_lossy()
                .to_string()
        );
        // APP_DATA_PATH is set from env var in prod to determine where pfm data to be stored.
        // set APP_DATA_PATH to path to pfm, e.g. /home/myuser/pfm, or /Users/myuser/pfm
        let location = std::env::var("APP_DATA_PATH").unwrap_or(default_location);
        let dir_name = "pfm-data";
        let storage_dir_path = PathBuf::from(location).join(dir_name);
        return storage_dir_path;
    }
});

/// Alias for ServerFS, Filesystem for storing data at server side.
pub type StorageFS = Arc<RwLock<ServerFS>>;

#[derive(Debug, Clone)]
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
    let root_pb = STORAGE_FS_DIR_PATH.clone();

    let root = utils::set_root(root_pb, STORAGE_FS_PERMISSION)
        .context("global: failed initializing storage fs")?;

    let latest = utils::set_sub_dir(&root, STORAGE_FS_LATEST_DIR_NAME, STORAGE_FS_PERMISSION)
        .context("global: failed initializing latest storage fs")?;

    let historical =
        utils::set_sub_dir(&root, STORAGE_FS_HISTORICAL_DIR_NAME, STORAGE_FS_PERMISSION)
            .context("global: failed initializing historical storage fs")?;

    let storage_fs = Arc::new(RwLock::new(ServerFS {
        root,
        latest,
        historical,
    }));

    Ok(storage_fs)
}

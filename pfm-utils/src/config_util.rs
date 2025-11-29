use configrs::config::Config as configrs;
use serde::Deserialize;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};

use anyhow::anyhow;

const ERROR_PREFIX: &str = "[utils]";

/// get config from env variables, or from .env file in workspace root for local dev.
pub fn get_config<CFG>(prefix: &'static str) -> Result<CFG, anyhow::Error>
where
    CFG: for<'de> Deserialize<'de> + Debug + Clone,
{
    let cfg = configrs::new().with_env_prefix(prefix);
    if cfg!(debug_assertions) {
        let workspace_dir = find_workspace_root()?;
        let dev_config_file = workspace_dir.join(".env");
        let config_builder = if dev_config_file.exists() {
            cfg.with_overwrite().with_env(&dev_config_file)
        } else {
            cfg
        };
        let ret = config_builder.build::<CFG>().map_err(|err| {
            anyhow!(
                "{} failed reading local config at {:?}: {}",
                ERROR_PREFIX,
                dev_config_file.as_path(),
                err
            )
        });
        return ret;
    }

    let ret = cfg
        .build::<CFG>()
        .map_err(|err| anyhow!("{} failed parsing env: {}", ERROR_PREFIX, err));

    ret
}

pub fn find_workspace_root() -> Result<PathBuf, anyhow::Error> {
    let mut current_dir = std::env::current_dir()?;

    loop {
        // Check for workspace root indicators
        if is_workspace_root(&current_dir) {
            return Ok(current_dir);
        }

        // Move up to parent directory
        if !current_dir.pop() {
            return Err(anyhow!("Root workspace not found"));
        }
    }
}

fn is_workspace_root(dir: &Path) -> bool {
    if dir.join(".git").is_dir() {
        return true;
    }

    let cargo_toml = dir.join("Cargo.toml");
    if cargo_toml.is_file() {
        if let Ok(contents) = std::fs::read_to_string(cargo_toml) {
            if contents.contains("[workspace]") {
                return true;
            }
        }
    }

    false
}

pub fn set_root(root: PathBuf, permission: u32) -> Result<PathBuf, anyhow::Error> {
    let is_exist = root.try_exists()?;

    if !is_exist {
        fs::create_dir_all(&root)?;
    }

    let metadata = fs::metadata(&root)?;
    let mut new_permissions = metadata.permissions();
    new_permissions.set_mode(permission);
    fs::set_permissions(&root, new_permissions)?;

    Ok(root)
}

pub fn set_sub_dir(
    parent: &PathBuf,
    sub_dir_name: &str,
    permission: u32,
) -> Result<PathBuf, anyhow::Error> {
    let sub_dir = parent.join(sub_dir_name);

    let is_exist = sub_dir.try_exists()?;

    if !is_exist {
        fs::create_dir_all(&sub_dir)?;
    }

    let metadata = fs::metadata(&sub_dir)?;
    let mut new_permissions = metadata.permissions();
    new_permissions.set_mode(permission);
    fs::set_permissions(&sub_dir, new_permissions)?;

    Ok(sub_dir)
}

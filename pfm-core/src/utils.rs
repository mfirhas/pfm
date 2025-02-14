use std::path::{Path, PathBuf};

use anyhow::anyhow;

pub fn find_file_in_workspace(filename: &str) -> Option<PathBuf> {
    let mut current_dir = std::env::current_dir().ok()?;

    let mut workspace_root = None;

    loop {
        if is_workspace_root(&current_dir) {
            workspace_root = Some(current_dir.clone());
        }

        let file_path = current_dir.join(filename);
        if file_path.is_file() {
            if let Some(root) = &workspace_root {
                if file_path.starts_with(root) {
                    return Some(file_path);
                }
            } else {
                return Some(file_path);
            }
        }

        if !current_dir.pop() {
            break;
        }
    }

    None
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

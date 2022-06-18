use anyhow::Context;
use directories::ProjectDirs;
use std::path::*;

pub const PROJECT_NAME: &str = "ocipkg";

/// Project root data directory
pub fn data_dir() -> anyhow::Result<PathBuf> {
    let p = ProjectDirs::from("", PROJECT_NAME, PROJECT_NAME)
        .context("System does not provide valid $HOME path")?;
    let dir = p.data_dir();
    Ok(dir.to_owned())
}

use crate::ImageName;
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

/// Create data directory for each image
pub fn image_dir(name: &ImageName) -> anyhow::Result<PathBuf> {
    let dir = data_dir()?;
    Ok(dir.join(format!(
        "{}/{}/__{}",
        name.hostname, name.name, name.reference
    )))
}

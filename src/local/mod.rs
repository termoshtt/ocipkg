use crate::{error::*, ImageName};
use directories::ProjectDirs;
use std::path::*;

pub const PROJECT_NAME: &str = "ocipkg";

/// Project root data directory
pub fn data_dir() -> Result<PathBuf> {
    let p = ProjectDirs::from("", PROJECT_NAME, PROJECT_NAME).ok_or(Error::NoValidHomeDirecotry)?;
    let dir = p.data_dir();
    Ok(dir.to_owned())
}

/// Create data directory for each image
pub fn image_dir(name: &ImageName) -> Result<PathBuf> {
    let dir = data_dir()?;
    if let Some(port) = name.port {
        Ok(dir.join(format!(
            "{}__{}/{}/__{}",
            name.hostname, port, name.name, name.reference
        )))
    } else {
        Ok(dir.join(format!(
            "{}/{}/__{}",
            name.hostname, name.name, name.reference
        )))
    }
}

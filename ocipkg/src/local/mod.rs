//! Manage container images stored in local storage

use crate::{
    distribution::{Name, Reference},
    image::Image,
    ImageName,
};
use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use std::{path::*, sync::OnceLock};

pub const DEFAULT_PROJECT_NAME: &str = "ocipkg";

static PROJECT_DIRS: OnceLock<ProjectDirs> = OnceLock::new();

pub fn set_project_dirs(dirs: ProjectDirs) -> Result<()> {
    PROJECT_DIRS
        .set(dirs)
        .map_err(|_| anyhow!("Failed to set project dirs"))
}

/// Project root data directory
pub fn data_dir() -> Result<PathBuf> {
    // FIXME: use `get_or_try_init` after it is stabilized
    let p = PROJECT_DIRS.get_or_init(|| {
        ProjectDirs::from("", DEFAULT_PROJECT_NAME, DEFAULT_PROJECT_NAME)
            .expect("No valid home directory")
    });
    let dir = p.data_dir();
    Ok(dir.to_owned())
}

/// Resolve a path to local storage where the image will be stored
pub fn image_dir(name: &ImageName) -> Result<PathBuf> {
    Ok(data_dir()?.join(name.as_path()))
}

fn path_to_image_name(path: &Path) -> Result<ImageName> {
    let rel_path = path
        .strip_prefix(data_dir()?)
        .expect("WalkDir must return path under data_dir");
    ImageName::from_path(rel_path)
}

/// Get images stored in local storage
pub fn get_image_list() -> Result<Vec<ImageName>> {
    let data_dir = data_dir()?;
    if !data_dir.exists() {
        return Ok(Vec::new());
    }

    let mut images = Vec::new();
    for entry in walkdir::WalkDir::new(data_dir) {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry
            .file_name()
            .to_str()
            .expect("Non UTF-8 path is never created in data directory");
        if name.starts_with("__") {
            images.push(path_to_image_name(path)?);
        }
    }
    Ok(images)
}

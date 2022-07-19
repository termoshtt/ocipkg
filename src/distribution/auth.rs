use crate::error::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, io, path::*};

/// Authentication info stored in filesystem
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StoredAuth {
    auths: HashMap<String, Auth>,
}

impl StoredAuth {
    /// Load authentication info stored by ocipkg
    pub fn load() -> Result<Self> {
        let mut auth = StoredAuth::default();
        if let Some(path) = auth_path() {
            auth.append(&path)?;
        }
        Ok(auth)
    }

    /// Load authentication info with docker and podman setting
    pub fn load_all() -> Result<Self> {
        let mut auth = StoredAuth::default();
        if let Some(path) = docker_auth_path() {
            auth.append(&path)?;
        }
        if let Some(path) = podman_auth_path() {
            auth.append(&path)?;
        }
        if let Some(path) = auth_path() {
            auth.append(&path)?;
        }
        Ok(auth)
    }

    pub fn insert(&mut self, domain: &str, octet: String) {
        self.auths.insert(domain.to_string(), Auth { auth: octet });
    }

    pub fn save(&self) -> Result<()> {
        let path = auth_path().ok_or(Error::NoValidRuntimeDirectory)?;
        if !path.parent().unwrap().exists() {
            fs::create_dir_all(&path)?;
        }
        let f = fs::File::create(&path)?;
        serde_json::to_writer_pretty(f, self)?;
        Ok(())
    }

    fn append(&mut self, path: &Path) -> Result<()> {
        let other = Self::from_path(path)?;
        for (key, value) in other.auths.into_iter() {
            self.auths.insert(key, value);
        }
        Ok(())
    }

    fn from_path(path: &Path) -> Result<Self> {
        if path.is_file() {
            let f = fs::File::open(path)?;
            Ok(serde_json::from_reader(io::BufReader::new(f))?)
        } else {
            Ok(Self::default())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Auth {
    auth: String,
}

fn auth_path() -> Option<PathBuf> {
    let dirs = directories::ProjectDirs::from("", "", "ocipkg")?;
    Some(dirs.runtime_dir()?.join("auth.json"))
}

fn docker_auth_path() -> Option<PathBuf> {
    let dirs = directories::BaseDirs::new()?;
    Some(dirs.home_dir().join(".docker/config.json"))
}

fn podman_auth_path() -> Option<PathBuf> {
    let dirs = directories::ProjectDirs::from("", "", "containers")?;
    Some(dirs.runtime_dir()?.join("auth.json"))
}

use crate::error::*;
use oci_spec::distribution::ErrorResponse;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, io, path::*};
use url::Url;

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

    /// Get token for using in API call
    ///
    /// Returns `None` if no authentication is required.
    pub fn get_token(&self, url: &url::Url) -> Result<Option<String>> {
        let test_url = url.join("/v2/").unwrap();
        let www_auth = match ureq::get(test_url.as_str()).call() {
            Ok(_) => return Ok(None),
            Err(ureq::Error::Status(status, res)) => {
                if status == 401 {
                    res.header("www-authenticate").unwrap().to_string()
                } else {
                    let err = res.into_json::<ErrorResponse>()?;
                    return Err(Error::RegistryError(err));
                }
            }
            Err(ureq::Error::Transport(e)) => return Err(Error::NetworkError(e)),
        };

        let (ty, realm) = parse_www_authenticate_header(&www_auth);
        if ty != "Bearer" {
            log::warn!("Unsupported authenticate type, fallback: {}", ty);
            return Ok(None);
        }
        let (token_url, query) = parse_bearer_realm(realm)?;

        let domain = token_url
            .domain()
            .expect("www-authenticate header returns invalid URL");
        if let Some(auth) = self.auths.get(domain) {
            let mut req = ureq::get(token_url.as_str())
                .set("Authorization", &format!("Basic {}", auth.auth))
                .set("Accept", "application/json");
            for (k, v) in query {
                req = req.query(k, v);
            }
            match req.call() {
                Ok(res) => {
                    let token = res.into_json::<Token>()?;
                    Ok(Some(token.token))
                }
                Err(ureq::Error::Status(..)) => Err(Error::AuthorizationFailed(url.clone())),
                Err(ureq::Error::Transport(e)) => Err(Error::NetworkError(e)),
            }
        } else {
            Ok(None)
        }
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

/// Parse the header of response. It must be in form:
///
/// ```text
/// WWW-Authenticate: <type> realm=<realm>
/// ```
///
/// https://developer.mozilla.org/en-US/docs/Web/HTTP/Authentication#www-authenticate_and_proxy-authenticate_headers
fn parse_www_authenticate_header(header: &str) -> (&str, &str) {
    let re = regex::Regex::new(r"(\w+) realm=(.+)").unwrap();
    let cap = re
        .captures(header)
        .expect("WWW-Authenticate header is invalid");
    let ty = cap.get(1).unwrap().as_str();
    let realm = cap.get(2).unwrap().as_str();
    (ty, realm)
}

/// Parse realm
///
/// XXX: Where this format is defined?
///
/// ghcr.io returns following:
///
/// ```text
/// Bearer realm="https://ghcr.io/token",service="ghcr.io",scope="repository:termoshtt/ocipkg/rust-lib:pull"
/// ```
fn parse_bearer_realm(realm: &str) -> Result<(Url, Vec<(&str, &str)>)> {
    let realm: Vec<_> = realm.split(',').collect();
    assert!(!realm.is_empty());
    let url = url::Url::parse(realm[0].trim_matches('"'))?;
    let query: Vec<_> = realm[1..]
        .iter()
        .map(|param| {
            let q: Vec<_> = param.split('=').collect();
            (q[0].trim_matches('"'), q[1].trim_matches('"'))
        })
        .collect();
    Ok((url, query))
}

#[derive(Deserialize)]
struct Token {
    token: String,
}

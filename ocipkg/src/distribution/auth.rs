use anyhow::{anyhow, Context, Result};
use base64::engine::{general_purpose::STANDARD, Engine};
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
            let new = Self::from_path(&path)?;
            auth.append(new)?;
        }
        Ok(auth)
    }

    /// Load authentication info with docker and podman setting
    pub fn load_all() -> Result<Self> {
        let mut auth = StoredAuth::default();
        if let Some(path) = docker_auth_path() {
            if let Ok(new) = Self::from_path(&path) {
                auth.append(new)?;
            }
        }
        if let Some(path) = podman_auth_path() {
            if let Ok(new) = Self::from_path(&path) {
                auth.append(new)?;
            }
        }
        if let Some(path) = auth_path() {
            let new = Self::from_path(&path)?;
            auth.append(new)?;
        }
        Ok(auth)
    }

    pub fn add(&mut self, domain: &str, username: &str, password: &str) {
        self.auths
            .insert(domain.to_string(), Auth::new(username, password));
    }

    #[deprecated(note = "Use `add` instead")]
    pub fn insert(&mut self, domain: &str, octet: String) {
        self.auths.insert(domain.to_string(), Auth { auth: octet });
    }

    pub fn save(&self) -> Result<()> {
        let path = auth_path().context("No valid runtime directory")?;
        let parent = path.parent().unwrap();
        if !parent.exists() {
            log::info!("Creating directory: {}", parent.display());
            fs::create_dir_all(parent)?;
        }
        log::info!("Saving auth info to: {}", path.display());
        let f = fs::File::create(&path)?;
        serde_json::to_writer_pretty(f, self)?;
        Ok(())
    }

    /// Get token by trying to access API root `/v2/`
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
                    return Err(err.into());
                }
            }
            Err(ureq::Error::Transport(e)) => return Err(e.into()),
        };

        let challenge = AuthChallenge::from_header(&www_auth)?;
        self.challenge(&challenge).map(Some)
    }

    /// Get token based on WWW-Authentication header
    pub fn challenge(&self, challenge: &AuthChallenge) -> Result<String> {
        let token_url = Url::parse(&challenge.url)?;
        let domain = token_url
            .domain()
            .expect("www-authenticate header returns invalid URL");

        let mut req = ureq::get(token_url.as_str()).set("Accept", "application/json");
        if let Some(auth) = self.auths.get(domain) {
            req = req.set("Authorization", &format!("Basic {}", auth.auth))
        }
        req = req
            .query("scope", &challenge.scope)
            .query("service", &challenge.service);
        let res = req.call()?;
        let token = res.into_json::<Token>()?;
        Ok(token.token)
    }

    pub fn append(&mut self, other: Self) -> Result<()> {
        for (key, value) in other.auths.into_iter() {
            if value.is_valid() {
                self.auths.insert(key, value);
            }
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
    // base64 encoded username:password
    auth: String,
}

impl Auth {
    fn new(username: &str, password: &str) -> Self {
        let auth = format!("{}:{}", username, password);
        let auth = STANDARD.encode(auth.as_bytes());
        Self { auth }
    }

    fn is_valid(&self) -> bool {
        let Ok(decoded) = STANDARD.decode(&self.auth) else {
            return false;
        };
        decoded.split(|b| *b == b':').count() == 2
    }
}

fn auth_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "ocipkg")
        .and_then(|dirs| Some(dirs.runtime_dir()?.join("auth.json")))
        .or_else(|| {
            // Most of container does not set XDG_RUNTIME_DIR,
            // and then this fallback to `~/.ocipkg/config.json` like docker.
            let dirs = directories::BaseDirs::new()?;
            Some(dirs.home_dir().join(".ocipkg/config.json"))
        })
}

fn docker_auth_path() -> Option<PathBuf> {
    let dirs = directories::BaseDirs::new()?;
    Some(dirs.home_dir().join(".docker/config.json"))
}

fn podman_auth_path() -> Option<PathBuf> {
    let dirs = directories::ProjectDirs::from("", "", "containers")?;
    Some(dirs.runtime_dir()?.join("auth.json"))
}

/// WWW-Authentication challenge
///
/// ```
/// use ocipkg::distribution::AuthChallenge;
///
/// let auth = AuthChallenge::from_header(
///   r#"Bearer realm="https://ghcr.io/token",service="ghcr.io",scope="repository:termoshtt/ocipkg/rust-lib:pull""#,
/// ).unwrap();
///
/// assert_eq!(auth, AuthChallenge {
///   url: "https://ghcr.io/token".to_string(),
///   service: "ghcr.io".to_string(),
///   scope: "repository:termoshtt/ocipkg/rust-lib:pull".to_string(),
/// });
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthChallenge {
    pub url: String,
    pub service: String,
    pub scope: String,
}

impl AuthChallenge {
    pub fn from_header(header: &str) -> Result<Self> {
        let err = || anyhow!("Unsupported WWW-Authenticate header: {}", header);
        let (ty, realm) = header.split_once(' ').ok_or_else(err)?;
        if ty != "Bearer" {
            return Err(err());
        }

        let mut url = None;
        let mut service = None;
        let mut scope = None;
        for param in realm.split(',') {
            let (key, value) = param.split_once('=').ok_or_else(err)?;
            let value = value.trim_matches('"').to_string();
            match key {
                "realm" => url = Some(value),
                "service" => service = Some(value),
                "scope" => scope = Some(value),
                _ => continue,
            }
        }
        Ok(Self {
            url: url.ok_or_else(err)?,
            service: service.ok_or_else(err)?,
            scope: scope.ok_or_else(err)?,
        })
    }
}

#[derive(Deserialize)]
struct Token {
    token: String,
}

use anyhow::{anyhow, bail, Context, Result};
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
        Self::from_path(&auth_path()?)
    }

    /// Load authentication info with docker and podman setting
    pub fn load_all() -> Result<Self> {
        let mut auth = None;
        for path in [docker_auth_path(), podman_auth_path(), auth_path()]
            .into_iter()
            .filter_map(|x| x.ok())
        {
            if let Ok(new) = Self::from_path(&path) {
                log::info!("Loaded auth info from: {}", path.display());
                auth.get_or_insert_with(|| Self::default()).append(new);
            }
        }
        auth.context("No valid auth info found")
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
    pub fn get_token(&self, url: &url::Url) -> Result<Option<String>> {
        let test_url = url.join("/v2/").unwrap();
        let challenge = match ureq::get(test_url.as_str()).call() {
            Ok(_) => return Ok(None),
            Err(e) => AuthChallenge::try_from(e)?,
        };
        self.challenge(&challenge).map(Some)
    }

    /// Get token based on WWW-Authentication header
    pub fn challenge(&self, challenge: &AuthChallenge) -> Result<String> {
        let token_url = Url::parse(&challenge.url)?;
        let domain = token_url
            .domain()
            .with_context(|| format!("www-authenticate header returns invalid URL: {token_url}"))?;

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

    pub fn append(&mut self, other: Self) {
        for (key, value) in other.auths.into_iter() {
            if value.is_valid() {
                self.auths.insert(key, value);
            }
        }
    }

    /// Load auth info from file
    pub fn from_path(path: &Path) -> Result<Self> {
        if !path.is_file() {
            bail!("Auth file not found: {}", path.display());
        }
        let f = fs::File::open(path)?;
        let loaded = serde_json::from_reader(io::BufReader::new(f))?;
        let mut out = Self::default();
        out.append(loaded);
        Ok(out)
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

fn home_dir() -> Result<PathBuf> {
    let dirs = directories::BaseDirs::new().context("Cannot get $HOME directory")?;
    Ok(dirs.home_dir().to_path_buf())
}

fn auth_path() -> Result<PathBuf> {
    let dirs = directories::ProjectDirs::from("", "", "ocipkg")
        .context("Cannot get project directory of ocipkg")?;
    if let Some(runtime_dir) = dirs.runtime_dir() {
        return Ok(runtime_dir.join("auth.json"));
    } else {
        // Most of container does not set XDG_RUNTIME_DIR,
        // and then this fallback to `~/.ocipkg/config.json` like docker.
        Ok(home_dir()?.join(".ocipkg/config.json"))
    }
}

fn docker_auth_path() -> Result<PathBuf> {
    Ok(home_dir()?.join(".docker/config.json"))
}

fn podman_auth_path() -> Result<PathBuf> {
    let dirs = directories::ProjectDirs::from("", "", "containers")
        .context("Cannot get the project directory of podman")?;
    Ok(dirs
        .runtime_dir()
        .context("Cannot get runtime directory of podman")?
        .join("auth.json"))
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

impl TryFrom<ureq::Error> for AuthChallenge {
    type Error = anyhow::Error;
    fn try_from(res: ureq::Error) -> Result<Self> {
        match res {
            ureq::Error::Status(status, res) => {
                if status == 401 && res.has("www-authenticate") {
                    Self::from_header(res.header("www-authenticate").unwrap())
                } else {
                    let err = res.into_json::<ErrorResponse>()?;
                    Err(err.into())
                }
            }
            ureq::Error::Transport(e) => Err(e.into()),
        }
    }
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

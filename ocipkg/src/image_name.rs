use crate::{Name, Reference};
use anyhow::{anyhow, bail, Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    path::{Path, PathBuf},
    str::FromStr,
};
use url::Url;

/// Image name
///
/// OCI defines the following grammar for "org.opencontainers.image.ref.name"
/// in the pre-defined annotation keys in [OCI image spec]:
///
/// ```text
/// ref       ::= component ("/" component)*
/// component ::= alphanum (separator alphanum)*
/// alphanum  ::= [A-Za-z0-9]+
/// separator ::= [-._:@+] | "--"
/// ```
///
/// The leading component is validated against this grammar, with `:` reserved
/// for the optional port. The remaining parts are validated by [Name] and
/// [Reference] according to the OCI Distribution Specification.
///
/// The hostname/name/reference split and defaults below are ocipkg conventions;
/// the OCI annotation grammar does not require its first component to be a DNS
/// hostname. In particular, `test_repo/testing:tag1` is a valid reference.
///
/// [OCI image spec]: https://github.com/opencontainers/image-spec/blob/v1.1.1/annotations.md#pre-defined-annotation-keys
///
/// Terminology
/// ------------
/// ocipkg calls each component of an image name:
///
/// ```text
/// ghcr.io/termoshtt/ocipkg/testing:latest
/// ^^^^^^^---------------------------------- hostname
///         ^^^^^^^^^^^^^^^^^^^^^^^^--------- name
///                                  ^^^^^^-- reference
/// ```
///
/// ```rust
/// use ocipkg::{ImageName, Name, Reference};
/// let name = ImageName::parse("ghcr.io/termoshtt/ocipkg/testing:latest")?;
/// assert_eq!(
///     name,
///     ImageName {
///         hostname: "ghcr.io".to_string(),
///         port: None,
///         name: Name::new("termoshtt/ocipkg/testing")?,
///         reference: Reference::new("latest")?,
///     }
/// );
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// If a port number is included:
///
/// ```text
/// localhost:5000/test_repo:tag1
/// ^^^^^^^^^---------------------- hostname
///           ^^^^----------------- port
///                ^^^^^^^^^------- name
///                          ^^^^-- reference
/// ```
///
/// ```
/// use ocipkg::{ImageName, Name, Reference};
/// let name = ImageName::parse("localhost:5000/test_repo:latest")?;
/// assert_eq!(
///     name,
///     ImageName {
///         hostname: "localhost".to_string(),
///         port: Some(5000),
///         name: Name::new("test_repo")?,
///         reference: Reference::new("latest")?,
///     }
/// );
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// [Reference] can be a digest:
///
/// ```text
/// quay.io/jitesoft/alpine:sha256:6755355f801f8e3694bffb1a925786813462cea16f1ce2b0290b6a48acf2500c
/// ^^^^^^^-------------------- hostname
///         ^^^^^^^^^^^^^^^---- name
///            reference ---^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
/// ```
///
/// ```
/// use ocipkg::{ImageName, Name, Reference};
/// let name = ImageName::parse("quay.io/jitesoft/alpine:sha256:6755355f801f8e3694bffb1a925786813462cea16f1ce2b0290b6a48acf2500c")?;
/// assert_eq!(
///     name,
///     ImageName {
///         hostname: "quay.io".to_string(),
///         port: None,
///         name: Name::new("jitesoft/alpine")?,
///         reference: Reference::new("sha256:6755355f801f8e3694bffb1a925786813462cea16f1ce2b0290b6a48acf2500c")?,
///     }
/// );
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// Default values
/// ---------------
/// If `hostname` is absent, use `registry-1.docker.io` for docker compatibility:
///
/// ```
/// use ocipkg::{ImageName, Name, Reference};
/// let name = ImageName::parse("ubuntu:20.04")?;
/// assert_eq!(
///     name,
///     ImageName {
///         hostname: "registry-1.docker.io".to_string(),
///         port: None,
///         name: Name::new("ubuntu")?,
///         reference: Reference::new("20.04")?,
///     }
/// );
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// If `reference` is absent, use `latest`:
///
/// ```
/// use ocipkg::{ImageName, Name, Reference};
/// let name = ImageName::parse("alpine").unwrap();
/// assert_eq!(
///     name,
///     ImageName {
///         hostname: "registry-1.docker.io".to_string(),
///         port: None,
///         name: Name::new("alpine")?,
///         reference: Reference::new("latest")?,
///     }
/// );
/// # Ok::<(), anyhow::Error>(())
/// ```
///
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImageName {
    /// Leading OCI reference component, excluding the optional port.
    /// This is not restricted to DNS hostname syntax.
    pub hostname: String,
    pub port: Option<u16>,
    pub name: Name,
    pub reference: Reference,
}

lazy_static::lazy_static! {
    // OCI image-spec v1.1.1, annotations.md: component ::= alphanum
    // (separator alphanum)*. ':' is represented separately by `port`.
    static ref HOSTNAME_RE: Regex =
        Regex::new(r"\A[A-Za-z0-9]+(([-._@+]|--)[A-Za-z0-9]+)*\z").unwrap();
}

impl fmt::Display for ImageName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(port) = self.port {
            write!(
                f,
                "{}:{}/{}:{}",
                self.hostname, port, self.name, self.reference
            )
        } else {
            write!(f, "{}/{}:{}", self.hostname, self.name, self.reference)
        }
    }
}

impl Default for ImageName {
    fn default() -> Self {
        Self::parse(&format!("{}", uuid::Uuid::new_v4().as_hyphenated()))
            .expect("UUID hyphenated must be valid name")
    }
}

impl FromStr for ImageName {
    type Err = anyhow::Error;
    fn from_str(name: &str) -> Result<Self> {
        let (hostname, name) = name
            .split_once('/')
            .unwrap_or(("registry-1.docker.io", name));
        let (hostname, port) = if let Some((hostname, port)) = hostname.split_once(':') {
            (hostname, Some(str::parse(port)?))
        } else {
            (hostname, None)
        };
        let (name, reference) = name.split_once(':').unwrap_or((name, "latest"));
        let image_name = ImageName {
            hostname: hostname.to_string(),
            port,
            name: Name::new(name)?,
            reference: Reference::new(reference)?,
        };
        image_name.validate_hostname()?;
        Ok(image_name)
    }
}

impl Serialize for ImageName {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ImageName {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ImageName::parse(&s).map_err(serde::de::Error::custom)
    }
}

impl ImageName {
    // Local storage must revalidate public fields at the filesystem boundary,
    // so hostname validation is shared with the top-level `local` module.
    pub(crate) fn validate_hostname(&self) -> Result<()> {
        if !HOSTNAME_RE.is_match(&self.hostname) {
            bail!(
                "Invalid leading OCI reference component {:?}: expected alphanumeric groups separated by '-', '--', '.', '_', '@', or '+'; specify a port separately",
                self.hostname
            );
        }
        Ok(())
    }

    pub fn parse(name: &str) -> Result<Self> {
        Self::from_str(name)
    }

    /// URL for OCI distribution API endpoint
    pub fn registry_url(&self) -> Result<Url> {
        let hostname = if let Some(port) = self.port {
            format!("{}:{}", self.hostname, port)
        } else {
            self.hostname.clone()
        };
        let url = if self.hostname.starts_with("localhost") {
            format!("http://{}", hostname)
        } else {
            format!("https://{}", hostname)
        };
        Ok(Url::parse(&url)?)
    }

    pub fn as_escaped_path(&self) -> PathBuf {
        PathBuf::from(if let Some(port) = &self.port {
            format!(
                "{}%3A{}/{}%3A{}",
                self.hostname,
                port,
                self.name,
                self.reference.encoded()
            )
        } else {
            format!(
                "{}/{}%3A{}",
                self.hostname,
                self.name.as_str(),
                self.reference.encoded()
            )
        })
    }

    pub fn from_escaped_path(path: &Path) -> Result<Self> {
        let image_name =
            urlencoding::decode(path.as_os_str().to_str().context("Non UTF-8 file path")?)?;
        Self::parse(&image_name)
    }

    /// Encode image name into a path by `{hostname}/{name}/__{reference}` or `{hostname}__{port}/{name}/__{reference}` if port is specified.
    pub fn as_path(&self) -> PathBuf {
        let reference = self.reference.replace(':', "__");
        PathBuf::from(if let Some(port) = self.port {
            format!("{}__{}/{}/__{}", self.hostname, port, self.name, reference)
        } else {
            format!("{}/{}/__{}", self.hostname, self.name, reference)
        })
    }

    /// Parse image name from a path encoded by [ImageName::as_path]
    pub fn from_path(path: &Path) -> Result<Self> {
        let components = path
            .components()
            .map(|c| {
                c.as_os_str()
                    .to_str()
                    .context("Try to convert a path including non UTF-8 character")
            })
            .collect::<Result<Vec<&str>>>()?;
        let n = components.len();
        if n < 3 {
            bail!(
                "Path for image name must consist of registry, name, and tag: {}",
                path.display()
            );
        }

        let registry = &components[0];
        let mut iter = registry.split("__");
        let hostname = iter
            .next()
            .with_context(|| anyhow!("Invalid registry: {registry}"))?
            .to_string();
        let port = iter
            .next()
            .map(|port| str::parse(port).context("Invalid port number"))
            .transpose()?;

        let name = Name::new(&components[1..n - 1].join("/"))?;

        let reference = Reference::new(
            &components[n - 1]
                .strip_prefix("__")
                .with_context(|| anyhow!("Missing tag in path: {}", path.display()))?
                .replace("__", ":"),
        )?;

        let image_name = ImageName {
            hostname,
            port,
            name,
            reference,
        };
        image_name.validate_hostname()?;
        Ok(image_name)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn reject_invalid_hostnames() {
        for hostname in [
            "",
            ".",
            "..",
            "...",
            "..\\outside",
            "host\\outside",
            "host ",
            "host\n",
            "host..name",
            "host__name",
            "host---name",
            "-host",
            "host-",
        ] {
            let input = format!("{hostname}/image:latest");
            assert!(ImageName::parse(&input).is_err(), "accepted {input:?}");
            let json = serde_json::to_string(&input).unwrap();
            assert!(serde_json::from_str::<ImageName>(&json).is_err());
        }
        for path in [
            "../image/__latest",
            "/image/__latest",
            "..\\outside/image/__latest",
        ] {
            assert!(
                ImageName::from_path(Path::new(path)).is_err(),
                "accepted {path:?}"
            );
        }
        assert!(ImageName::from_escaped_path(Path::new("%2E%2E/image%3Alatest")).is_err());
    }

    #[test]
    fn valid_hostnames() -> Result<()> {
        for hostname in [
            "localhost",
            "registry-1.docker.io",
            "127.0.0.1",
            "Registry.EXAMPLE.com",
            "test_repo",
            "test+repo",
            "test@repo",
            "test--repo",
        ] {
            for port in ["", ":5000"] {
                let input = format!("{hostname}{port}/example/image:latest");
                let name = ImageName::parse(&input)?;
                assert_eq!(name.to_string(), input);
                assert_eq!(ImageName::from_path(&name.as_path())?, name);
                assert_eq!(ImageName::from_escaped_path(&name.as_escaped_path())?, name);
                let json = serde_json::to_string(&name)?;
                assert_eq!(serde_json::from_str::<ImageName>(&json)?, name);
            }
        }
        Ok(())
    }

    #[test]
    fn as_path() -> Result<()> {
        fn test(name: &str, path: &Path) -> Result<()> {
            let image_name = ImageName::parse(name)?;
            assert_eq!(image_name.as_path(), path);
            assert_eq!(ImageName::from_path(&image_name.as_path())?, image_name);
            Ok(())
        }

        test(
            "localhost:5000/test_repo:latest",
            "localhost__5000/test_repo/__latest".as_ref(),
        )?;
        test(
            "ubuntu:20.04",
            "registry-1.docker.io/ubuntu/__20.04".as_ref(),
        )?;
        test("alpine", "registry-1.docker.io/alpine/__latest".as_ref())?;
        test(
            "quay.io/jitesoft/alpine:sha256:6755355f801f8e3694bffb1a925786813462cea16f1ce2b0290b6a48acf2500c",
            "quay.io/jitesoft/alpine/__sha256__6755355f801f8e3694bffb1a925786813462cea16f1ce2b0290b6a48acf2500c".as_ref()
        )?;
        Ok(())
    }

    #[test]
    fn escaped_path() -> Result<()> {
        fn test(name: &str, path: &Path) -> Result<()> {
            let image_name = ImageName::parse(name)?;
            let escaped = image_name.as_escaped_path();
            assert_eq!(escaped, path);
            assert_eq!(ImageName::from_escaped_path(&escaped)?, image_name);
            Ok(())
        }

        test(
            "localhost:5000/test_repo:latest",
            "localhost%3A5000/test_repo%3Alatest".as_ref(),
        )?;
        test(
            "ubuntu:20.04",
            "registry-1.docker.io/ubuntu%3A20.04".as_ref(),
        )?;
        test("alpine", "registry-1.docker.io/alpine%3Alatest".as_ref())?;
        test(
            "quay.io/jitesoft/alpine:sha256:6755355f801f8e3694bffb1a925786813462cea16f1ce2b0290b6a48acf2500c",
            "quay.io/jitesoft/alpine%3Asha256%3A6755355f801f8e3694bffb1a925786813462cea16f1ce2b0290b6a48acf2500c".as_ref()
        )?;
        Ok(())
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct SerializeTest {
        name: ImageName,
    }

    #[test]
    fn serialize() {
        let input = SerializeTest {
            name: ImageName::parse("localhost:5000/test_repo:latest").unwrap(),
        };
        let json = serde_json::to_string(&input).unwrap();
        assert_eq!(json, r#"{"name":"localhost:5000/test_repo:latest"}"#);

        let output: SerializeTest = serde_json::from_str(&json).unwrap();
        assert_eq!(input, output)
    }
}

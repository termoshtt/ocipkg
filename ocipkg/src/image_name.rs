use crate::distribution::{Name, Reference};
use anyhow::{anyhow, bail, Context, Result};
use std::{
    fmt,
    path::{Path, PathBuf},
};
use url::Url;

/// Image name
///
/// The input must be valid both as "org.opencontainers.image.ref.name"
/// defined in pre-defined annotation keys in [OCI image spec]:
///
/// ```text
/// ref       ::= component ("/" component)*
/// component ::= alphanum (separator alphanum)*
/// alphanum  ::= [A-Za-z0-9]+
/// separator ::= [-._:@+] | "--"
/// ```
///
/// and as an argument for [docker tag].
///
/// [OCI image spec]: https://github.com/opencontainers/image-spec/blob/main/annotations.md#pre-defined-annotation-keys
/// [docker tag]: https://docs.docker.com/engine/reference/commandline/tag/
///
/// Terminology
/// ------------
/// We call each components of image name to match OCI distribution spec:
///
/// ```text
/// ghcr.io/termoshtt/ocipkg/testing:latest
/// ^^^^^^^---------------------------------- hostname
///         ^^^^^^^^^^^^^^^^^^^^^^^^--------- name
///                                  ^^^^^^-- reference
/// ```
///
/// ```rust
/// use ocipkg::{ImageName, distribution::{Name, Reference}};
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
/// use ocipkg::{ImageName, distribution::{Name, Reference}};
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
/// Default values
/// ---------------
/// If `hostname` is absent, use `registry-1.docker.io` for docker compatibility:
///
/// ```
/// use ocipkg::{ImageName, distribution::{Name, Reference}};
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
/// use ocipkg::{ImageName, distribution::{Name, Reference}};
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
    pub hostname: String,
    pub port: Option<u16>,
    pub name: Name,
    pub reference: Reference,
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

impl ImageName {
    pub fn parse(name: &str) -> Result<Self> {
        let (hostname, name) = name
            .split_once('/')
            .unwrap_or(("registry-1.docker.io", name));
        let (hostname, port) = if let Some((hostname, port)) = hostname.split_once(':') {
            (hostname, Some(str::parse(port)?))
        } else {
            (hostname, None)
        };
        let (name, reference) = name.split_once(':').unwrap_or((name, "latest"));
        Ok(ImageName {
            hostname: hostname.to_string(),
            port,
            name: Name::new(name)?,
            reference: Reference::new(reference)?,
        })
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

        Ok(ImageName {
            hostname,
            port,
            name,
            reference,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ttlsh_style() {
        let image_name =
            ImageName::parse("ttl.sh/79219A62-4E86-41B3-854D-95D8F4636C9C:1h").unwrap();
        assert_eq!(image_name.hostname, "ttl.sh".to_string(),);
        assert_eq!(image_name.port, None);
        assert_eq!(
            image_name.name.as_str(),
            "79219A62-4E86-41B3-854D-95D8F4636C9C"
        );
        assert_eq!(image_name.reference.as_str(), "1h")
    }

    fn test_as_path(name: &str, path: &Path) -> Result<()> {
        let image_name = ImageName::parse(name)?;
        assert_eq!(image_name.as_path(), path);
        assert_eq!(ImageName::from_path(&image_name.as_path())?, image_name);
        Ok(())
    }

    #[test]
    fn as_path() -> Result<()> {
        test_as_path(
            "localhost:5000/test_repo:latest",
            "localhost__5000/test_repo/__latest".as_ref(),
        )?;
        test_as_path(
            "ubuntu:20.04",
            "registry-1.docker.io/ubuntu/__20.04".as_ref(),
        )?;
        test_as_path("alpine", "registry-1.docker.io/alpine/__latest".as_ref())?;
        test_as_path(
            "quay.io/jitesoft/alpine:sha256:6755355f801f8e3694bffb1a925786813462cea16f1ce2b0290b6a48acf2500c",
            "quay.io/jitesoft/alpine/__sha256__6755355f801f8e3694bffb1a925786813462cea16f1ce2b0290b6a48acf2500c".as_ref()
        )?;
        Ok(())
    }
}

use crate::{
    distribution::{Name, Reference},
    error::*,
};
use std::fmt;
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
/// # Ok::<(), ocipkg::error::Error>(())
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
/// # Ok::<(), ocipkg::error::Error>(())
/// ```
///
/// Default values
/// ---------------
/// If `hostname` is absent, use `registry-1.docker.io` for docker compatiblity:
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
/// # Ok::<(), ocipkg::error::Error>(())
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
/// # Ok::<(), ocipkg::error::Error>(())
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
}

#[cfg(test)]
mod test {
    use crate::ImageName;

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
}

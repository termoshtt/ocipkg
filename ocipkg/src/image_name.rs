use crate::distribution::{Name, Reference};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImageName {
    pub domain: String,
    pub port: Option<u16>,
    pub name: Name,
    pub reference: Reference,
}

impl ImageName {
    pub fn parse(name: &str) -> anyhow::Result<Self> {
        let (domain, name) = name.split_once('/').unwrap_or(("docker.io", name));
        let (domain, port) = if let Some((domain, port)) = domain.split_once(':') {
            (domain, Some(str::parse(port)?))
        } else {
            (domain, None)
        };
        let (name, reference) = name.split_once(':').unwrap_or((name, "latest"));
        Ok(ImageName {
            domain: domain.to_string(),
            port,
            name: Name::new(name)?,
            reference: Reference::new(reference)?,
        })
    }

    /// URL for OCI distribution API endpoint
    pub fn registry_url(&self) -> anyhow::Result<Url> {
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
    use super::*;

    #[test]
    fn image_name() -> anyhow::Result<()> {
        let name = ImageName::parse("ghcr.io/termoshtt/ocipkg/testing:latest")?;
        assert_eq!(
            name,
            ImageName {
                domain: "ghcr.io".to_string(),
                port: None,
                name: Name::new("termoshtt/ocipkg/testing")?,
                reference: Reference::new("latest")?,
            }
        );

        let name = ImageName::parse("localhost:5000/test_repo:latest")?;
        assert_eq!(
            name,
            ImageName {
                domain: "localhost".to_string(),
                port: Some(5000),
                name: Name::new("test_repo")?,
                reference: Reference::new("latest")?,
            }
        );

        let name = ImageName::parse("ubuntu:20.04")?;
        assert_eq!(
            name,
            ImageName {
                domain: "docker.io".to_string(),
                port: None,
                name: Name::new("ubuntu")?,
                reference: Reference::new("20.04")?,
            }
        );

        let name = ImageName::parse("alpine").unwrap();
        assert_eq!(
            name,
            ImageName {
                domain: "docker.io".to_string(),
                port: None,
                name: Name::new("alpine")?,
                reference: Reference::new("latest")?,
            }
        );

        Ok(())
    }
}

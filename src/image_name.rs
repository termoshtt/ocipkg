use crate::{Name, Reference};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImageName {
    pub domain: String,
    pub name: Name,
    pub reference: Reference,
}

impl ImageName {
    pub fn parse(name: &str) -> anyhow::Result<Self> {
        let (domain, name) = name.split_once('/').unwrap_or(("docker.io", name));
        let (name, reference) = name.split_once(':').unwrap_or((name, "latest"));
        Ok(ImageName {
            domain: domain.to_string(),
            name: Name::new(&name)?,
            reference: Reference::new(reference)?,
        })
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
                name: Name::new("termoshtt/ocipkg/testing")?,
                reference: Reference::new("latest")?,
            }
        );

        let name = ImageName::parse("ubuntu:20.04")?;
        assert_eq!(
            name,
            ImageName {
                domain: "docker.io".to_string(),
                name: Name::new("ubuntu")?,
                reference: Reference::new("20.04")?,
            }
        );

        let name = ImageName::parse("alpine").unwrap();
        assert_eq!(
            name,
            ImageName {
                domain: "docker.io".to_string(),
                name: Name::new("alpine")?,
                reference: Reference::new("latest")?,
            }
        );

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImageName {
    pub url: String,
    pub name: String,
    pub reference: String,
}

impl ImageName {
    pub fn new(name: &str) -> anyhow::Result<Self> {
        let (domain, name) = name.split_once('/').unwrap_or(("docker.io", name));
        let (name, reference) = name.split_once(':').unwrap_or((name, "latest"));
        let url = if domain.starts_with("localhost") {
            format!("http://{}", domain)
        } else {
            format!("https://{}", domain)
        };
        Ok(ImageName {
            url,
            name: name.to_string(),
            reference: reference.to_string(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn image_name() {
        let name = ImageName::new("ghcr.io/termoshtt/ocipkg/testing:latest").unwrap();
        assert_eq!(
            name,
            ImageName {
                url: "https://ghcr.io".to_string(),
                name: "termoshtt/ocipkg/testing".to_string(),
                reference: "latest".to_string(),
            }
        );

        let name = ImageName::new("ubuntu:20.04").unwrap();
        assert_eq!(
            name,
            ImageName {
                url: "https://docker.io".to_string(),
                name: "ubuntu".to_string(),
                reference: "20.04".to_string(),
            }
        );

        let name = ImageName::new("alpine").unwrap();
        assert_eq!(
            name,
            ImageName {
                url: "https://docker.io".to_string(),
                name: "alpine".to_string(),
                reference: "latest".to_string(),
            }
        );
    }
}

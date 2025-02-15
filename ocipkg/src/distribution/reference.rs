use anyhow::{bail, Result};
use oci_spec::image::Digest;
use regex::Regex;
use std::{fmt, str::FromStr};

/// Reference of container image stored in the repository
///
/// In [OCI distribution spec](https://github.com/opencontainers/distribution-spec/blob/main/spec.md):
/// > `<reference>` MUST be either (a) the digest of the manifest or (b) a tag
/// > `<reference>` as a tag MUST be at most 128 characters
/// > in length and MUST match the following regular expression:
/// > ```text
/// > [a-zA-Z0-9_][a-zA-Z0-9._-]{0,127}
/// > ```
/// This struct checks this restriction at creation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Reference(String);

impl std::ops::Deref for Reference {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Reference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

lazy_static::lazy_static! {
    static ref REF_RE: Regex = Regex::new(r"^[a-zA-Z0-9_][a-zA-Z0-9._-]{0,127}$").unwrap();
}

impl Reference {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Encode upper letters and `:` to URL encoding, e.g. `A` -> `%41`
    pub fn encoded(&self) -> String {
        self.0
            .chars()
            .map(|c| {
                if c.is_ascii_uppercase() || c == ':' {
                    format!("%{:02X}", c as u8)
                } else {
                    c.to_string()
                }
            })
            .collect()
    }

    pub fn new(name: &str) -> Result<Self> {
        if REF_RE.is_match(name) {
            Ok(Reference(name.to_string()))
        } else if name.contains(':') {
            _ = Digest::from_str(name)?;
            Ok(Reference(name.to_string()))
        } else {
            bail!("Invalid reference {name}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference() {
        assert_eq!(Reference::new("latest").unwrap().as_str(), "latest");
        assert_eq!(
            Reference::new(
                "sha256:a1b2c3d4a1b2c3d4a1b2c3d4a1b2c3d4a1b2c3d4a1b2c3d4a1b2c3d4a1b2c3d4"
            )
            .unwrap()
            .as_str(),
            "sha256:a1b2c3d4a1b2c3d4a1b2c3d4a1b2c3d4a1b2c3d4a1b2c3d4a1b2c3d4a1b2c3d4"
        );
        // @ is not allowed
        assert!(Reference::new("my_super_tag@2").is_err());

        // Upper ASCII is encoded
        assert_eq!(
            Reference::new("SuperTag").unwrap().encoded(),
            "%53uper%54ag"
        );
        assert_eq!(
            Reference::new(
                "sha256:a1b2c3d4a1b2c3d4a1b2c3d4a1b2c3d4a1b2c3d4a1b2c3d4a1b2c3d4a1b2c3d4"
            )
            .unwrap()
            .encoded(),
            "sha256%3Aa1b2c3d4a1b2c3d4a1b2c3d4a1b2c3d4a1b2c3d4a1b2c3d4a1b2c3d4a1b2c3d4"
        );
    }
}

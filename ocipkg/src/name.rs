use anyhow::{bail, Result};
use regex::Regex;
use std::fmt;

/// Namespace of the repository
///
/// The name must satisfy the following regular expression in [OCI distribution spec 1.1.0](https://github.com/opencontainers/distribution-spec/blob/v1.1.0/spec.md):
///
/// ```regex
/// [a-z0-9]+((\.|_|__|-+)[a-z0-9]+)*(\/[a-z0-9]+((\.|_|__|-+)[a-z0-9]+)*)*
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name(String);

impl std::ops::Deref for Name {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

lazy_static::lazy_static! {
    static ref NAME_RE: Regex = Regex::new(r"^[a-z0-9]+((\.|_|__|-+)[a-z0-9]+)*(\/[a-z0-9]+((\.|_|__|-+)[a-z0-9]+)*)*$").unwrap();
}

impl Name {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn new(name: &str) -> Result<Self> {
        if NAME_RE.is_match(name) {
            Ok(Name(name.to_string()))
        } else {
            bail!("Invalid name: {name}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name() {
        assert_eq!(Name::new("ghcr.io").unwrap().as_str(), "ghcr.io");
        // Head must be alphanum
        assert!(Name::new("_ghcr.io").is_err());
        assert!(Name::new("/ghcr.io").is_err());

        // Capital letter is not allowed
        assert!(Name::new("ghcr.io/Termoshtt").is_err());
    }
}

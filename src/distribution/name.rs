use crate::error::*;
use derive_more::Deref;
use regex::Regex;
use std::fmt;

/// Namespace of the repository
///
/// In [OCI distribution spec](https://github.com/opencontainers/distribution-spec/blob/main/spec.md):
/// > `<name>` MUST match the following regular expression:
/// > ```text
/// > [a-z0-9]+([._-][a-z0-9]+)*(/[a-z0-9]+([._-][a-z0-9]+)*)*
/// > ```
/// This struct checks this restriction at creation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deref)]
pub struct Name(String);

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

lazy_static::lazy_static! {
    static ref NAME_RE: Regex = Regex::new(r"^[a-z0-9]+([._-][a-z0-9]+)*(/[a-z0-9]+([._-][a-z0-9]+)*)*$").unwrap();
}

impl Name {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn new(name: &str) -> Result<Self> {
        if NAME_RE.is_match(name) {
            Ok(Name(name.to_string()))
        } else {
            Err(Error::InvalidName(name.to_string()))
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
    }
}

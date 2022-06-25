//! Binding to OCI distribution spec

use derive_more::Deref;
use regex::Regex;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error<'a> {
    #[error("Invalid <name> of repository: {0}")]
    InvalidRepositoryName(&'a str),

    #[error("Invalid reference: {0}")]
    InvalidReference(&'a str),
}

pub struct Client {}

/// Namespace of the repository
///
/// In [OCI distribution spec](https://github.com/opencontainers/distribution-spec/blob/main/spec.md):
/// > `<name>` MUST match the following regular expression:
/// > ```text
/// > [a-z0-9]+([._-][a-z0-9]+)*(/[a-z0-9]+([._-][a-z0-9]+)*)*
/// > ```
/// This struct checks this restriction at creation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deref)]
pub struct Name<'a>(&'a str);

lazy_static::lazy_static! {
    static ref NAME_RE: Regex = Regex::new(r"^[a-z0-9]+([._-][a-z0-9]+)*(/[a-z0-9]+([._-][a-z0-9]+)*)*$").unwrap();
}

impl<'a> Name<'a> {
    pub fn as_str(&self) -> &str {
        self.0
    }

    pub fn from_str(name: &'a str) -> Result<Self, Error<'a>> {
        if NAME_RE.is_match(name) {
            Ok(Name(name))
        } else {
            Err(Error::InvalidRepositoryName(name))
        }
    }
}

/// Reference of container image stored in the repository
///
/// In [OCI distribution spec](https://github.com/opencontainers/distribution-spec/blob/main/spec.md):
/// > `<reference>` as a tag MUST be at most 128 characters
/// > in length and MUST match the following regular expression:
/// > ```text
/// > [a-zA-Z0-9_][a-zA-Z0-9._-]{0,127}
/// > ```
/// This struct checks this restriction at creation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deref)]
pub struct Reference<'a>(&'a str);

lazy_static::lazy_static! {
    static ref REF_RE: Regex = Regex::new(r"^[a-zA-Z0-9_][a-zA-Z0-9._-]{0,127}$").unwrap();
}

impl<'a> Reference<'a> {
    pub fn as_str(&self) -> &str {
        self.0
    }

    pub fn from_str(name: &'a str) -> Result<Self, Error<'a>> {
        if REF_RE.is_match(name) {
            Ok(Reference(name))
        } else {
            Err(Error::InvalidReference(name))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name() {
        assert_eq!(Name::from_str("ghcr.io").unwrap().as_str(), "ghcr.io");
        // Head must be alphanum
        assert!(Name::from_str("_ghcr.io").is_err());
        assert!(Name::from_str("/ghcr.io").is_err());
    }

    #[test]
    fn reference() {
        assert_eq!(Name::from_str("latest").unwrap().as_str(), "latest");
        // @ is not allowed
        assert!(Name::from_str("my_super_tag@2").is_err());
    }
}

use derive_more::Deref;
use regex::Regex;

use crate::error::Error;

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

    pub fn new(name: &'a str) -> Result<Self, Error<'a>> {
        if NAME_RE.is_match(name) {
            Ok(Name(name))
        } else {
            Err(Error::InvalidRepositoryName(name))
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

use derive_more::Deref;
use regex::Regex;

use crate::error::Error;

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

    pub fn new(name: &'a str) -> Result<Self, Error<'a>> {
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
    fn reference() {
        assert_eq!(Reference::new("latest").unwrap().as_str(), "latest");
        // @ is not allowed
        assert!(Reference::new("my_super_tag@2").is_err());
    }
}

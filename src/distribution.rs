//! Binding to [OCI distribution spec](https://github.com/opencontainers/distribution-spec)

use derive_more::Deref;
use regex::Regex;
use serde::Deserialize;
use thiserror::Error;
use url::Url;

/// Error occured while handling distribution API
#[derive(Debug, Error)]
pub enum Error<'a> {
    #[error("Invalid <name> of repository: {0}")]
    InvalidRepositoryName(&'a str),

    #[error("Invalid reference: {0}")]
    InvalidReference(&'a str),

    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),

    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
}

/// A client for `/v2/<name>/` API endpoint
pub struct Client<'a> {
    client: reqwest::Client,
    /// URL to registry server
    url: Url,
    /// Name of repository
    name: Name<'a>,
}

/// Response of `/v2/<name>/tags/list`
#[derive(Debug, Clone, PartialEq, Deserialize)]
struct TagList {
    name: String,
    tags: Vec<String>,
}

impl<'a> Client<'a> {
    pub fn new(url: &str, name: &'a str) -> Result<Self, Error<'a>> {
        let client = reqwest::Client::new();
        let url = Url::parse(url)?;
        let name = Name::new(name)?;
        Ok(Client { client, url, name })
    }

    pub async fn get_tags(&self) -> Result<Vec<String>, Error<'a>> {
        let url = self
            .url
            .join(&format!("/v2/{}/tags/list", self.name.as_str()))?;
        let tag_list = self.client.get(url).send().await?.json::<TagList>().await?;
        Ok(tag_list.tags)
    }
}

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
    fn name() {
        assert_eq!(Name::new("ghcr.io").unwrap().as_str(), "ghcr.io");
        // Head must be alphanum
        assert!(Name::new("_ghcr.io").is_err());
        assert!(Name::new("/ghcr.io").is_err());
    }

    #[test]
    fn reference() {
        assert_eq!(Name::new("latest").unwrap().as_str(), "latest");
        // @ is not allowed
        assert!(Name::new("my_super_tag@2").is_err());
    }

    //
    // Following tests need registry server. See test/fixture.sh for setting.
    // These tests are ignored by default.
    //

    const TEST_URL: &str = "http://localhost:5000";
    const TEST_REPO: &str = "test_repo";

    #[tokio::test]
    #[ignore]
    async fn get_tags() -> anyhow::Result<()> {
        let client = Client::new(TEST_URL, TEST_REPO)?;
        let tags = client.get_tags().await?;
        assert_eq!(
            tags,
            &["tag1".to_string(), "tag2".to_string(), "tag3".to_string()]
        );
        Ok(())
    }
}

//! Binding to [OCI distribution spec](https://github.com/opencontainers/distribution-spec)

use serde::Deserialize;
use url::Url;

use crate::{error::Error, Name};

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

    /// Get tags of `<name>` repository.
    ///
    /// ```text
    /// GET /v2/<name>/tags/list
    /// ```
    ///
    /// See [corresponding OCI distribution spec document](https://github.com/opencontainers/distribution-spec/blob/main/spec.md#content-discovery) for detail.
    pub async fn get_tags(&self) -> Result<Vec<String>, Error<'a>> {
        let url = self
            .url
            .join(&format!("/v2/{}/tags/list", self.name.as_str()))?;
        let tag_list = self.client.get(url).send().await?.json::<TagList>().await?;
        Ok(tag_list.tags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

//! Binding to [OCI distribution spec](https://github.com/opencontainers/distribution-spec)

use bytes::Bytes;
use oci_spec::image::*;
use serde::Deserialize;
use url::Url;

use crate::{Digest, Name, Reference};

/// A client for `/v2/<name>/` API endpoint
pub struct Client {
    client: reqwest::Client,
    /// URL to registry server
    url: Url,
    /// Name of repository
    name: Name,
}

/// Response of `/v2/<name>/tags/list`
#[derive(Debug, Clone, PartialEq, Deserialize)]
struct TagList {
    name: String,
    tags: Vec<String>,
}

impl Client {
    pub fn new(url: &str, name: &str) -> anyhow::Result<Self> {
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
    pub async fn get_tags(&self) -> anyhow::Result<Vec<String>> {
        let tag_list = self
            .client
            .get(
                self.url
                    .join(&format!("/v2/{}/tags/list", self.name.as_str()))?,
            )
            .send()
            .await?
            .json::<TagList>()
            .await?;
        Ok(tag_list.tags)
    }

    /// Get manifest for given repository
    ///
    /// ```text
    /// GET /v2/<name>/manifests/<reference>
    /// ```
    ///
    /// See [corresponding OCI distribution spec document](https://github.com/opencontainers/distribution-spec/blob/main/spec.md#pulling-manifests) for detail.
    pub async fn get_manifest(&self, reference: &str) -> anyhow::Result<ImageManifest> {
        let reference = Reference::new(reference)?;
        let manifest = self
            .client
            .get(self.url.join(&format!(
                "/v2/{}/manifests/{}",
                self.name.as_str(),
                reference.as_str()
            ))?)
            .header("Accept", MediaType::ImageManifest.to_docker_v2s2()?)
            .send()
            .await?
            .text()
            .await?;
        let manifest = ImageManifest::from_reader(manifest.as_bytes())?;
        Ok(manifest)
    }

    /// Get blob for given digest
    ///
    /// ```text
    /// GET /v2/<name>/blobs/<digest>
    /// ```
    ///
    /// See [corresponding OCI distribution spec document](https://github.com/opencontainers/distribution-spec/blob/main/spec.md#pulling-blobs) for detail.
    pub async fn get_blob(&self, digest: &str) -> anyhow::Result<Bytes> {
        let digest = Digest::new(digest)?;
        let blob = self
            .client
            .get(self.url.join(&format!(
                "/v2/{}/blobs/{}",
                self.name.as_str(),
                format!("{}", digest)
            ))?)
            .send()
            .await?
            .bytes()
            .await?;
        Ok(blob)
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

    #[tokio::test]
    #[ignore]
    async fn get_images() -> anyhow::Result<()> {
        let client = Client::new(TEST_URL, TEST_REPO)?;
        for tag in ["tag1", "tag2", "tag3"] {
            let manifest = client.get_manifest(tag).await?;
            for layer in manifest.layers() {
                let buf = client.get_blob(layer.digest()).await?;
                dbg!(buf.len());
            }
        }
        Ok(())
    }
}

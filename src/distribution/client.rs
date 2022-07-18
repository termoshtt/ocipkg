use bytes::Bytes;
use oci_spec::{distribution::*, image::*};
use url::Url;

use crate::{distribution::*, error::*, Digest};

/// A client for `/v2/<name>/` API endpoint
pub struct Client {
    client: reqwest::Client,
    agent: ureq::Agent,
    /// URL to registry server
    url: Url,
    /// Name of repository
    name: Name,
}

impl Client {
    pub fn new(url: &Url, name: &str) -> Result<Self> {
        let client = reqwest::Client::new();
        let name = Name::new(name)?;
        Ok(Client {
            client,
            agent: ureq::Agent::new(),
            url: url.clone(),
            name,
        })
    }

    /// Get tags of `<name>` repository.
    ///
    /// ```text
    /// GET /v2/<name>/tags/list
    /// ```
    ///
    /// See [corresponding OCI distribution spec document](https://github.com/opencontainers/distribution-spec/blob/main/spec.md#content-discovery) for detail.
    pub fn get_tags(&self) -> Result<Vec<String>> {
        let url = self.url.join(&format!("/v2/{}/tags/list", self.name))?;
        let res = self.agent.get(url.as_str()).call()?;
        if res.status() == 200 {
            let tag_list = res.into_json::<TagList>()?;
            Ok(tag_list.tags().to_vec())
        } else {
            let err = res.into_json::<ErrorResponse>()?;
            Err(Error::RegistryError(err))
        }
    }

    /// Get manifest for given repository
    ///
    /// ```text
    /// GET /v2/<name>/manifests/<reference>
    /// ```
    ///
    /// See [corresponding OCI distribution spec document](https://github.com/opencontainers/distribution-spec/blob/main/spec.md#pulling-manifests) for detail.
    pub fn get_manifest(&self, reference: &str) -> Result<ImageManifest> {
        let reference = Reference::new(reference)?;
        let url = self
            .url
            .join(&format!("/v2/{}/manifests/{}", self.name, reference))?;
        let res = self
            .agent
            .get(url.as_str())
            .set(
                "Accept",
                MediaType::ImageManifest
                    .to_docker_v2s2()
                    .expect("Never fails since ImageManifest is supported"),
            )
            .call()?;
        if res.status() == 200 {
            let manifest = ImageManifest::from_reader(res.into_reader())?;
            Ok(manifest)
        } else {
            let err = res.into_json::<ErrorResponse>()?;
            Err(Error::RegistryError(err))
        }
    }

    /// Push manifest to registry
    ///
    /// ```text
    /// PUT /v2/<name>/manifests/<reference>
    /// ```
    ///
    /// Manifest must be pushed after blobs are updated.
    ///
    /// See [corresponding OCI distribution spec document](https://github.com/opencontainers/distribution-spec/blob/main/spec.md#pushing-manifests) for detail.
    pub async fn push_manifest(&self, reference: &str, manifest: &ImageManifest) -> Result<Url> {
        let reference = Reference::new(reference)?;
        let mut buf = Vec::new();
        manifest.to_writer(&mut buf)?;
        let res = self
            .client
            .put(
                self.url
                    .join(&format!("/v2/{}/manifests/{}", self.name, reference))?,
            )
            .header("Content-Type", MediaType::ImageManifest.to_string())
            .body(buf)
            .send()
            .await?;
        let url = response_with_location(res).await?;
        Ok(url)
    }

    /// Get blob for given digest
    ///
    /// ```text
    /// GET /v2/<name>/blobs/<digest>
    /// ```
    ///
    /// See [corresponding OCI distribution spec document](https://github.com/opencontainers/distribution-spec/blob/main/spec.md#pulling-blobs) for detail.
    pub async fn get_blob(&self, digest: &str) -> Result<Bytes> {
        let digest = Digest::new(digest)?;
        let res = self
            .client
            .get(
                self.url
                    .join(&format!("/v2/{}/blobs/{}", self.name.as_str(), digest,))?,
            )
            .send()
            .await?;
        if res.status().is_success() {
            let blob = res.bytes().await?;
            Ok(blob)
        } else {
            let err = res.json::<ErrorResponse>().await?;
            Err(Error::RegistryError(err))
        }
    }

    /// Push blob to registry
    ///
    /// ```text
    /// POST /v2/<name>/blobs/uploads/
    /// ```
    ///
    /// and following `PUT` to URL obtained by `POST`.
    ///
    /// See [corresponding OCI distribution spec document](https://github.com/opencontainers/distribution-spec/blob/main/spec.md#pushing-manifests) for detail.
    pub async fn push_blob(&self, blob: &[u8]) -> Result<Url> {
        let res = self
            .client
            .post(
                self.url
                    .join(&format!("/v2/{}/blobs/uploads/", self.name))?,
            )
            .send()
            .await?;
        let url = response_with_location(res).await?;

        let digest = Digest::from_buf_sha256(blob);
        let res = self
            .client
            .put(url.clone())
            .query(&[("digest", digest.to_string())])
            .header("Content-Length", blob.len())
            .header("Content-Type", "application/octet-stream")
            .body(blob.to_vec())
            .send()
            .await?;
        let url = response_with_location(res).await?;
        Ok(url)
    }
}

// Most of API returns `Location: <location>`
async fn response_with_location(res: reqwest::Response) -> Result<Url> {
    if res.status().is_success() {
        let location = res
            .headers()
            .get("Location")
            .expect("Location header is lacked, invalid response of OCI registry");
        Ok(Url::parse(
            location
                .to_str()
                .expect("Invalid charactor in OCI registry response"),
        )?)
    } else {
        let err = res.json::<ErrorResponse>().await?;
        Err(Error::RegistryError(err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //
    // Following tests need registry server. See test/fixture.sh for setting.
    // These tests are ignored by default.
    //

    fn test_url() -> Url {
        Url::parse("http://localhost:5000").unwrap()
    }
    const TEST_REPO: &str = "test_repo";

    #[tokio::test]
    #[ignore]
    async fn get_tags() -> Result<()> {
        let client = Client::new(&test_url(), TEST_REPO)?;
        let mut tags = client.get_tags()?;
        tags.sort_unstable();
        assert_eq!(
            tags,
            &["tag1".to_string(), "tag2".to_string(), "tag3".to_string()]
        );
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn get_images() -> Result<()> {
        let client = Client::new(&test_url(), TEST_REPO)?;
        for tag in ["tag1", "tag2", "tag3"] {
            let manifest = client.get_manifest(tag)?;
            for layer in manifest.layers() {
                let buf = client.get_blob(layer.digest()).await?;
                dbg!(buf.len());
            }
        }
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn push_blob() -> Result<()> {
        let client = Client::new(&test_url(), TEST_REPO)?;
        let url = client.push_blob("test string".as_bytes()).await?;
        dbg!(url);
        Ok(())
    }
}

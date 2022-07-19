use oci_spec::{distribution::*, image::*};
use url::Url;

use crate::{distribution::*, error::*, Digest};

/// A client for `/v2/<name>/` API endpoint
pub struct Client {
    agent: ureq::Agent,
    /// URL to registry server
    url: Url,
    /// Name of repository
    name: Name,
    /// Authorization token
    token: Option<String>,
}

impl Client {
    pub fn new(url: Url, name: Name) -> Result<Self> {
        let auth = StoredAuth::load_all()?;
        let token = auth.get_token(&url)?;
        Ok(Client {
            agent: ureq::Agent::new(),
            url,
            name,
            token,
        })
    }

    fn add_auth_header(&self, req: ureq::Request) -> ureq::Request {
        if let Some(token) = &self.token {
            req.set("Authorization", &format!("Bearer {}", token))
        } else {
            req
        }
    }

    fn get(&self, url: &Url) -> ureq::Request {
        self.add_auth_header(self.agent.get(url.as_str()))
    }

    fn put(&self, url: &Url) -> ureq::Request {
        self.add_auth_header(self.agent.put(url.as_str()))
    }

    fn post(&self, url: &Url) -> ureq::Request {
        self.add_auth_header(self.agent.post(url.as_str()))
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
        let res = self.get(&url).call().check_response()?;
        let tag_list = res.into_json::<TagList>()?;
        Ok(tag_list.tags().to_vec())
    }

    /// Get manifest for given repository
    ///
    /// ```text
    /// GET /v2/<name>/manifests/<reference>
    /// ```
    ///
    /// See [corresponding OCI distribution spec document](https://github.com/opencontainers/distribution-spec/blob/main/spec.md#pulling-manifests) for detail.
    pub fn get_manifest(&self, reference: &Reference) -> Result<ImageManifest> {
        let url = self
            .url
            .join(&format!("/v2/{}/manifests/{}", self.name, reference))?;
        let res = self
            .get(&url)
            .set(
                "Accept",
                MediaType::ImageManifest
                    .to_docker_v2s2()
                    .expect("Never fails since ImageManifest is supported"),
            )
            .call()
            .check_response()?;
        let manifest = ImageManifest::from_reader(res.into_reader())?;
        Ok(manifest)
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
    pub fn push_manifest(&self, reference: &Reference, manifest: &ImageManifest) -> Result<Url> {
        let mut buf = Vec::new();
        manifest.to_writer(&mut buf)?;
        let url = self
            .url
            .join(&format!("/v2/{}/manifests/{}", self.name, reference))?;
        let res = self
            .put(&url)
            .set("Content-Type", &MediaType::ImageManifest.to_string())
            .send_bytes(&buf)
            .check_response()?;
        Ok(Url::parse(res.header("Location").expect(
            "Location header is lacked in OCI registry response",
        ))?)
    }

    /// Get blob for given digest
    ///
    /// ```text
    /// GET /v2/<name>/blobs/<digest>
    /// ```
    ///
    /// See [corresponding OCI distribution spec document](https://github.com/opencontainers/distribution-spec/blob/main/spec.md#pulling-blobs) for detail.
    pub fn get_blob(&self, digest: &Digest) -> Result<Vec<u8>> {
        let url = self
            .url
            .join(&format!("/v2/{}/blobs/{}", self.name.as_str(), digest,))?;
        let res = self.get(&url).call().check_response()?;
        let mut bytes = Vec::new();
        res.into_reader().read_to_end(&mut bytes)?;
        Ok(bytes)
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
    pub fn push_blob(&self, blob: &[u8]) -> Result<Url> {
        let url = self
            .url
            .join(&format!("/v2/{}/blobs/uploads/", self.name))?;
        let res = self.post(&url).call().check_response()?;
        let url = Url::parse(
            res.header("Location")
                .expect("Location header is lacked in OCI registry response"),
        )?;

        let digest = Digest::from_buf_sha256(blob);
        let res = self
            .put(&url)
            .query("digest", &digest.to_string())
            .set("Content-Length", &blob.len().to_string())
            .set("Content-Type", "application/octet-stream")
            .send_bytes(blob)
            .check_response()?;
        Ok(Url::parse(res.header("Location").expect(
            "Location header is lacked in OCI registry response",
        ))?)
    }
}

trait CheckResponse {
    fn check_response(self) -> Result<ureq::Response>;
}

impl CheckResponse for std::result::Result<ureq::Response, ureq::Error> {
    fn check_response(self) -> Result<ureq::Response> {
        match self {
            Ok(res) => Ok(res),
            Err(ureq::Error::Status(status, res)) => {
                if status == 401 {
                    if let Some(msg) = res.header("www-authenticate") {
                        log::error!("Server returns WWW-Authenticate header: {}", msg);
                    }
                }
                let err = res.into_json::<ErrorResponse>()?;
                Err(Error::RegistryError(err))
            }
            Err(ureq::Error::Transport(e)) => Err(Error::NetworkError(e)),
        }
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
    fn test_name() -> Name {
        Name::new("test_repo").unwrap()
    }

    #[test]
    #[ignore]
    fn get_tags() -> Result<()> {
        let client = Client::new(test_url(), test_name())?;
        let mut tags = client.get_tags()?;
        tags.sort_unstable();
        assert_eq!(
            tags,
            &["tag1".to_string(), "tag2".to_string(), "tag3".to_string()]
        );
        Ok(())
    }

    #[test]
    #[ignore]
    fn get_images() -> Result<()> {
        let client = Client::new(test_url(), test_name())?;
        for tag in ["tag1", "tag2", "tag3"] {
            let manifest = client.get_manifest(&Reference::new(tag)?)?;
            for layer in manifest.layers() {
                let buf = client.get_blob(&Digest::new(layer.digest())?)?;
                dbg!(buf.len());
            }
        }
        Ok(())
    }

    #[test]
    #[ignore]
    fn push_blob() -> Result<()> {
        let client = Client::new(test_url(), test_name())?;
        let url = client.push_blob("test string".as_bytes())?;
        dbg!(url);
        Ok(())
    }
}

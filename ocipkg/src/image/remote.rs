use crate::{
    distribution::{Client, StoredAuth},
    image::{Image, ImageBuilder},
    ImageName,
};
use anyhow::Result;
use oci_spec::image::{Digest, ImageManifest};

/// An image stored in remote registry as [Image]
pub struct Remote {
    image_name: ImageName,
    client: Client,
}

impl Remote {
    pub fn new(image_name: ImageName) -> Result<Self> {
        let client = Client::from_image_name(&image_name)?;
        Ok(Self { image_name, client })
    }

    pub fn new_with_auth(image_name: ImageName, auth: StoredAuth) -> Result<Self> {
        let client = Client::from_image_name_with_auth(&image_name, auth)?;
        Ok(Self { image_name, client })
    }

    pub fn add_basic_auth(&mut self, domain: &str, username: &str, password: &str) {
        self.client.add_basic_auth(domain, username, password);
    }
}

impl Image for Remote {
    fn get_name(&mut self) -> Result<ImageName> {
        Ok(self.image_name.clone())
    }

    fn get_blob(&mut self, digest: &Digest) -> Result<Vec<u8>> {
        self.client.get_blob(digest)
    }

    fn get_manifest(&mut self) -> Result<ImageManifest> {
        self.client.get_manifest(&self.image_name.reference)
    }
}

/// Build a [Remote] image, pushing blobs and manifest to remote registry
pub struct RemoteBuilder {
    image_name: ImageName,
    client: Client,
}

impl RemoteBuilder {
    pub fn new(image_name: ImageName) -> Result<Self> {
        let client = Client::from_image_name(&image_name)?;
        Ok(Self { image_name, client })
    }

    pub fn new_with_auth(image_name: ImageName, auth: StoredAuth) -> Result<Self> {
        let client = Client::from_image_name_with_auth(&image_name, auth)?;
        Ok(Self { image_name, client })
    }

    pub fn add_basic_auth(&mut self, domain: &str, username: &str, password: &str) {
        self.client.add_basic_auth(domain, username, password);
    }
}

impl ImageBuilder for RemoteBuilder {
    type Image = Remote;

    fn add_blob(&mut self, data: &[u8]) -> Result<(Digest, u64)> {
        let (digest, _url) = self.client.push_blob(data)?;
        Ok((digest, data.len() as u64))
    }

    fn build(self, manifest: ImageManifest) -> Result<Self::Image> {
        self.client
            .push_manifest(&self.image_name.reference, &manifest)?;
        Ok(Remote {
            image_name: self.image_name,
            client: self.client,
        })
    }
}

use crate::{
    distribution::Client,
    image::{Image, ImageBuilder},
    Digest, ImageName,
};
use anyhow::Result;
use oci_spec::image::ImageManifest;

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
}

impl ImageBuilder for RemoteBuilder {
    type Image = Remote;

    fn add_blob(&mut self, data: &[u8]) -> Result<(Digest, i64)> {
        let (digest, _url) = self.client.push_blob(data)?;
        Ok((digest, data.len() as i64))
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

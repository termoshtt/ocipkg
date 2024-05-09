use crate::{distribution::Client, image::Image, Digest, ImageName};
use anyhow::Result;
use oci_spec::image::ImageManifest;

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

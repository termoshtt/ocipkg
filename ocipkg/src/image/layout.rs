use crate::{Digest, ImageName};
use anyhow::{Context, Result};
use oci_spec::image::{Descriptor, DescriptorBuilder, ImageIndex, ImageManifest, MediaType};

/// Handler of [OCI Image Layout] containing single manifest.
///
/// Though the [OCI Image Layout] allows containing multiple manifests in a single layout,
/// this trait assumes a single manifest in a single layout.
///
/// [OCI Image Layout]: https://github.com/opencontainers/image-spec/blob/v1.1.0/image-layout.md
///
pub trait ImageLayout {
    /// Get `index.json`
    fn get_index(&mut self) -> Result<ImageIndex>;
    /// Get blob content.
    fn get_blob(&mut self, digest: &Digest) -> Result<Vec<u8>>;

    /// Get manifest stored in the image layout.
    ///
    /// Note that this trait assumes a single manifest in a single layout.
    /// If `index.json` contains `org.opencontainers.image.ref.name` annotation, it is returned as [ImageName].
    fn get_manifest(&mut self) -> Result<(Option<ImageName>, ImageManifest)> {
        let index = self.get_index()?;
        let desc = index.manifests().first().context("Missing manifest")?;
        let name = if let Some(name) = desc
            .annotations()
            .as_ref()
            .and_then(|annotations| annotations.get("org.opencontainers.image.ref.name"))
        {
            // Invalid image name raises an error, while missing name is just ignored.
            Some(ImageName::parse(name)?)
        } else {
            None
        };
        let digest = Digest::from_descriptor(desc)?;
        let manifest = serde_json::from_slice(self.get_blob(&digest)?.as_slice())?;
        Ok((name, manifest))
    }
}

/// Create new image layout.
///
/// Creating [ImageManifest] is out of scope of this trait.
pub trait ImageLayoutBuilder {
    /// Handler of generated image.
    type ImageLayout: ImageLayout;
    /// Add a blob to the image layout.
    fn add_blob(&mut self, data: &[u8]) -> Result<(Digest, i64)>;
    /// Finish building image layout.
    fn build(self, manifest: ImageManifest, name: ImageName) -> Result<Self::ImageLayout>;

    /// A placeholder for `application/vnd.oci.empty.v1+json`
    fn add_empty_json(&mut self) -> Result<Descriptor> {
        let (digest, size) = self.add_blob(b"{}")?;
        Ok(DescriptorBuilder::default()
            .media_type(MediaType::EmptyJSON)
            .size(size)
            .digest(digest.to_string())
            .build()?)
    }
}

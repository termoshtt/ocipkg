use crate::Digest;
use anyhow::{Context, Result};
use oci_spec::image::{ImageIndex, ImageManifest};

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
    fn get_manifest(&mut self) -> Result<ImageManifest> {
        let index = self.get_index()?;
        let digest =
            Digest::from_descriptor(index.manifests().first().context("Missing manifest")?)?;
        Ok(serde_json::from_slice(self.get_blob(&digest)?.as_slice())?)
    }
}

/// Create new image layout.
///
/// Creating [ImageManifest] is out of scope of this trait.
pub trait ImageLayoutBuilder {
    /// Handler of generated image.
    type ImageLayout: ImageLayout;
    fn add_blob(&mut self, data: &[u8]) -> Result<Digest>;
    fn finish(self, manifest: ImageManifest) -> Result<Self::ImageLayout>;
}

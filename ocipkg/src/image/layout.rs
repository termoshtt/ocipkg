use crate::{error::*, oci_spec::image::ImageManifest, Digest};

/// Handler of [OCI Image Layout] containing single manifest.
///
/// Though the [OCI Image Layout] allows containing multiple manifests in a single layout,
/// this trait assumes a single manifest in a single layout.
///
/// [OCI Image Layout]: https://github.com/opencontainers/image-spec/blob/v1.1.0/image-layout.md
///
pub trait ImageLayout {
    /// Get manifest stored in the image layout.
    fn get_manifest(&self) -> Result<ImageManifest>;
    /// Get digest of blob stored in the image layout except the manifest.
    fn get_blobs(&self) -> Result<Vec<Digest>>;
    /// Get blob content.
    fn get_blob(&self, digest: &Digest) -> Result<Vec<u8>>;
}

/// Create new image layout.
///
/// See [ImageLayout] for detail.
pub trait ImageLayoutBuilder {
    /// Handler of generated image.
    type ImageLayout: ImageLayout;
    fn add_blob(&mut self, data: &[u8]) -> Result<Digest>;
    fn finish(self) -> Result<Self::ImageLayout>;
}

use crate::{Digest, ImageName};
use anyhow::{bail, Context, Result};
use oci_spec::image::{Descriptor, DescriptorBuilder, ImageIndex, ImageManifest, MediaType};

/// Handler of [OCI Image Layout] with containing single manifest and its name.
///
/// - [OCI Image Layout] allows empty image name, i.e. no `org.opencontainers.image.ref.name` annotation, but this trait does not allow it.
/// - [OCI Image Layout] allows containing multiple manifests in a single layout,
///   this trait assumes a single manifest in a single layout.
///
/// [OCI Image Layout]: https://github.com/opencontainers/image-spec/blob/v1.1.0/image-layout.md
///
pub trait Image {
    /// The name of this image.
    fn get_name(&mut self) -> Result<ImageName>;

    /// Get blob content.
    fn get_blob(&mut self, digest: &Digest) -> Result<Vec<u8>>;

    /// The manifest of this image
    fn get_manifest(&mut self) -> Result<ImageManifest>;
}

/// Create new image layout.
///
/// Creating [ImageManifest] is out of scope of this trait.
pub trait ImageBuilder {
    /// Handler of generated image.
    type Image: Image;
    /// Add a blob to the image layout.
    fn add_blob(&mut self, data: &[u8]) -> Result<(Digest, i64)>;
    /// Finish building image layout.
    fn build(self, manifest: ImageManifest, name: ImageName) -> Result<Self::Image>;

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

pub(crate) fn get_name_from_index(index: &ImageIndex) -> Result<ImageName> {
    if index.manifests().len() != 1 {
        bail!("Multiple manifests in a index.json, it is not allowed in ocipkg.");
    }
    let manifest = index.manifests().first().unwrap();
    let name = manifest
        .annotations()
        .as_ref()
        .and_then(|annotations| annotations.get("org.opencontainers.image.ref.name"))
        .context("org.opencontainers.image.ref.name is not found in manifest annotation")?;
    ImageName::parse(name)
}

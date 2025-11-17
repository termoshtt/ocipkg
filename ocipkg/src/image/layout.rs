use crate::{
    image::{OciArchive, OciDir},
    ImageName,
};

#[cfg(feature = "remote")]
use crate::image::Remote;
use anyhow::{bail, Context, Result};
use oci_spec::image::{
    Descriptor, DescriptorBuilder, Digest, ImageIndex, ImageManifest, MediaType,
};
use std::path::Path;

/// Handler of [OCI Image Layout] with containing single manifest
///
/// - [OCI Image Layout] allows containing multiple manifests in a single layout,
///   this trait assumes a single manifest in a single layout.
///
/// [OCI Image Layout]: https://github.com/opencontainers/image-spec/blob/v1.1.0/image-layout.md
///
pub trait Image {
    /// The name of this image. This fails if the image does not have name.
    fn get_name(&mut self) -> Result<ImageName>;

    /// Get blob content.
    fn get_blob(&mut self, digest: &Digest) -> Result<Vec<u8>>;

    /// The manifest of this image
    fn get_manifest(&mut self) -> Result<ImageManifest>;
}

/// Build an [Image]
///
/// Creating [ImageManifest] is out of scope of this trait.
pub trait ImageBuilder {
    /// Handler of generated image.
    type Image: Image;

    /// Add a blob to the image layout.
    fn add_blob(&mut self, data: &[u8]) -> Result<(Digest, u64)>;

    /// Finish building image layout.
    fn build(self, manifest: ImageManifest) -> Result<Self::Image>;

    /// A placeholder for `application/vnd.oci.empty.v1+json`
    fn add_empty_json(&mut self) -> Result<Descriptor> {
        let (digest, size) = self.add_blob(b"{}")?;
        Ok(DescriptorBuilder::default()
            .media_type(MediaType::EmptyJSON)
            .size(size)
            .digest(digest)
            .build()?)
    }
}

/// Copy image from one to another.
pub fn copy<From: Image, To: ImageBuilder>(from: &mut From, mut to: To) -> Result<To::Image> {
    let name = from.get_name()?;
    let manifest = from.get_manifest()?;
    for layer in manifest.layers() {
        let digest = layer.digest();
        let blob = from.get_blob(digest)?;
        let (digest_new, size) = to.add_blob(&blob)?;
        if digest != &digest_new {
            bail!("Digest of a layer in {name} mismatch: {digest} != {digest_new}",);
        }
        if size != layer.size() {
            bail!(
                "Size of a layer in {name} mismatch: {size} != {}",
                layer.size()
            );
        }
    }
    let config = manifest.config();
    let digest = config.digest();
    let blob = from.get_blob(digest)?;
    let (digest_new, size) = to.add_blob(&blob)?;
    if digest != &digest_new {
        bail!("Digest of a config in {name} mismatch: {digest} != {digest_new}",);
    }
    if size != config.size() {
        bail!(
            "Size of a config in {name} mismatch: {size} != {}",
            config.size()
        );
    }
    to.build(manifest)
}

pub fn read(name_or_path: &str) -> Result<Box<dyn Image>> {
    let path: &Path = name_or_path.as_ref();
    if path.is_file() {
        return Ok(Box::new(OciArchive::new(path)?));
    }
    if path.is_dir() {
        return Ok(Box::new(OciDir::new(path)?));
    }

    #[cfg(feature = "remote")]
    {
        if let Ok(image_name) = ImageName::parse(name_or_path) {
            return Ok(Box::new(Remote::new(image_name)?));
        }
        bail!("Invalid image name or path: {}", name_or_path);
    }

    #[cfg(not(feature = "remote"))]
    bail!("Invalid image name or path (remote feature disabled): {}", name_or_path);
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

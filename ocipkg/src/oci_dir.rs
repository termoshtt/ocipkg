//! oci-dir, i.e. a directory of local filesystem in the form of [OCI Image Layout specification](https://github.com/opencontainers/image-spec/blob/v1.1.0/image-layout.md)
//!
//! The name "oci-dir" comes from [`podman save`](https://docs.podman.io/en/latest/markdown/podman-save.1.html).
//! It is not defined in OCI Image specification.
//!

use crate::{error::*, Digest};
use oci_spec::image::{DescriptorBuilder, ImageIndexBuilder, ImageManifest, MediaType};
use std::{fs, path::PathBuf};

/// Builder for `.oci-dir` directory
///
/// This is responsible for saving any data and manifest files as blobs, and create `index.json` file.
///
pub struct OciDirBuilder {
    oci_dir_root: PathBuf,
}

impl OciDirBuilder {
    pub fn new(oci_dir_root: PathBuf) -> Result<Self> {
        if oci_dir_root.exists() {
            return Err(Error::ImageAlreadyExists(oci_dir_root));
        }
        fs::create_dir_all(&oci_dir_root)?;
        Ok(Self { oci_dir_root })
    }

    pub fn save_blob(&self, data: &[u8]) -> Result<Digest> {
        let digest = Digest::from_buf_sha256(data);
        let out = self.oci_dir_root.join(digest.as_path());
        fs::create_dir_all(out.parent().unwrap())?;
        fs::write(out, data)?;
        Ok(digest)
    }

    /// Create `index.json` file with image manifest.
    ///
    /// Although `index.json` can store multiple manifests, this API does not support it.
    pub fn finish(self, manifest: ImageManifest) -> Result<()> {
        let manifest_json = serde_json::to_string(&manifest)?;
        let digest = self.save_blob(manifest_json.as_bytes())?;
        let descriptor = DescriptorBuilder::default()
            .media_type(MediaType::ImageManifest)
            .size(manifest_json.len() as i64)
            .digest(digest.to_string())
            .build()?;
        let index = ImageIndexBuilder::default()
            .manifests(vec![descriptor])
            .build()?;
        fs::write(
            self.oci_dir_root.join("index.json"),
            serde_json::to_string(&index)?,
        )?;
        Ok(())
    }
}

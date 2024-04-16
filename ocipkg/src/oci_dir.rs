//! oci-dir handler

use crate::{error::*, Digest};
use oci_spec::image::{DescriptorBuilder, ImageIndexBuilder, ImageManifest, MediaType};
use std::{fs, path::PathBuf};

/// Builder for `.oci-dir` directory
///
/// This is responsible for saving any data and manifest files as blobs, and create `index.json` file.
///
pub struct LocalOciDirBuilder {
    oci_dir_root: PathBuf,
}

impl LocalOciDirBuilder {
    pub fn new(root: PathBuf) -> Result<Self> {
        if root.exists() {
            return Err(Error::ImageAlreadyExists(root));
        }
        let oci_dir_root = root.join(".oci-dir");
        fs::create_dir_all(&oci_dir_root)?;
        Ok(Self { oci_dir_root })
    }

    pub fn save_blob(&self, data: &[u8]) -> Result<Digest> {
        let digest = Digest::from_buf_sha256(data);
        fs::write(self.oci_dir_root.join(digest.as_path()), data)?;
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

//! oci-dir handler

use crate::{error::*, Digest};
use oci_spec::image::{
    Descriptor, DescriptorBuilder, ImageIndex, ImageIndexBuilder, ImageManifest, MediaType,
};
use serde::de;
use std::{fs, path::PathBuf};

/// Builder for `.oci-dir` directory
///
/// This is responsible for saving any data and manifest files as blobs, and create `index.json` file.
///
pub struct Builder {
    oci_dir_root: PathBuf,
}

impl Builder {
    pub fn new(root: PathBuf) -> Result<Self> {
        if root.exists() {
            return Err(Error::ImageAlreadyExists(root));
        }
        let oci_dir_root = root.join(".oci-dir");
        fs::create_dir_all(&oci_dir_root)?;
        Ok(Self { oci_dir_root })
    }

    pub fn save_blob(&self, media_type: MediaType, data: &[u8]) -> Result<Descriptor> {
        let digest = Digest::from_buf_sha256(data);
        fs::write(self.oci_dir_root.join(digest.as_path()), data)?;
        Ok(DescriptorBuilder::default()
            .digest(digest.to_string())
            .size(data.len() as i64)
            .media_type(media_type)
            .build()?)
    }

    /// Create `index.json` file with image manifest.
    ///
    /// This API does not support to save multiple manifest into single index.json.
    pub fn finish(self, manifest: ImageManifest) -> Result<()> {
        let manifest_json = serde_json::to_string(&manifest)?;
        let descriptor = self.save_blob(MediaType::ImageManifest, manifest_json.as_bytes())?;
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

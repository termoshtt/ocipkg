use crate::{error::*, local::image_dir, ImageName};
use chrono::Local;
use oci_spec::image::{self, Descriptor, MediaType};
use std::fs;

/// Writer trait based on [Guidelines for Artifact Usage](https://github.com/opencontainers/image-spec/blob/v1.1.0/manifest.md#guidelines-for-artifact-usage) of OCI image specification 1.1.0
pub trait ArtifactWriter {
    /// Add a blob with media-type to the image. This blob is added to layers.
    fn add_blob(&mut self, descriptor: Descriptor, blob: &[u8]) -> Result<()>;
    /// Add a config to the image manifest
    fn add_config(&mut self, descriptor: Descriptor, config: &[u8]) -> Result<()>;
    /// Add an annotation to the image manifest
    fn add_annotation(&mut self, key: &str, value: &str) -> Result<()>;
    /// Set `artifactType` field in the manifest
    fn set_artifact_type(&mut self, artifact_type: MediaType) -> Result<()>;
    /// Finish writing the image
    fn finish(self) -> Result<()>;
}

pub struct LocalArtifact {
    image_name: ImageName,
}

impl Drop for LocalArtifact {
    fn drop(&mut self) {
        if let Err(e) = self.cleanup() {
            log::error!("Failed to cleanup: {}", e);
        }
    }
}

impl LocalArtifact {
    pub fn new(image_name: ImageName) -> Result<Self> {
        let path = image_dir(&image_name)?;
        if path.exists() {
            return Err(Error::ImageAlreadyExists(path));
        }
        Ok(Self { image_name })
    }

    fn cleanup(&self) -> Result<()> {
        Ok(fs::remove_dir_all(self.image_name.local_path()?)?)
    }
}

pub struct ArchiveArtifact {}

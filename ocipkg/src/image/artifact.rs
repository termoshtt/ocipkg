use crate::{error::*, local::image_dir, Digest, ImageName};
use oci_spec::image::{Descriptor, ImageManifest, ImageManifestBuilder, MediaType};
use std::{fs, path::PathBuf};

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
    manifest: ImageManifest,
    oci_dir_root: PathBuf,
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
        let oci_dir_root = image_name.local_path()?.join(".oci-dir");
        fs::create_dir_all(&oci_dir_root)?;
        Ok(Self {
            image_name,
            manifest: ImageManifestBuilder::default().build()?,
            oci_dir_root,
        })
    }

    fn cleanup(&self) -> Result<()> {
        Ok(fs::remove_dir_all(self.image_name.local_path()?)?)
    }
}

impl ArtifactWriter for LocalArtifact {
    fn add_blob(&mut self, descriptor: Descriptor, blob: &[u8]) -> Result<()> {
        let digest = Digest::new(&descriptor.digest())?;
        // FIXME: check digest of blob
        let path = self.oci_dir_root.join(digest.as_path());
        fs::write(path, blob)?;
        self.manifest.layers_mut().push(descriptor);
        Ok(())
    }

    fn add_config(&mut self, descriptor: Descriptor, config: &[u8]) -> Result<()> {
        let digest = Digest::new(&descriptor.digest())?;
        // FIXME: check digest of config
        let path = self.oci_dir_root.join(digest.as_path());
        fs::write(path, config)?;
        self.manifest.set_config(descriptor);
        Ok(())
    }

    fn add_annotation(&mut self, key: &str, value: &str) -> Result<()> {
        self.manifest
            .annotations_mut()
            .get_or_insert(Default::default())
            .insert(key.to_string(), value.to_string());
        Ok(())
    }

    fn set_artifact_type(&mut self, artifact_type: MediaType) -> Result<()> {
        self.manifest.set_artifact_type(Some(artifact_type));
        Ok(())
    }

    fn finish(self) -> Result<()> {
        todo!()
    }
}

pub struct ArchiveArtifact {}

use crate::{error::*, local::image_dir, Digest, ImageName};
use oci_spec::image::{Descriptor, ImageManifest, ImageManifestBuilder, MediaType};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Create a new OCI artifact based on [Guidelines for Artifact Usage] of OCI image specification 1.1.0
///
/// The artifact is created on the local storage where [crate::local] manages as a [OCI Image layout].
///
/// [Guidelines for Artifact Usage]: https://github.com/opencontainers/image-spec/blob/v1.1.0/manifest.md#guidelines-for-artifact-usage
/// [OCI Image layout]: https://github.com/opencontainers/image-spec/blob/v1.1.0/image-layout.md
pub struct LocalArtifactBuilder {
    image_name: ImageName,
    manifest: ImageManifest,
    root: PathBuf,
}

impl LocalArtifactBuilder {
    pub fn new(image_name: ImageName) -> Result<Self> {
        let path = image_dir(&image_name)?;
        if path.exists() {
            return Err(Error::ImageAlreadyExists(path));
        }
        let root = image_name.local_path()?;
        fs::create_dir_all(&root)?;
        Ok(Self {
            image_name,
            manifest: ImageManifestBuilder::default().build()?,
            root,
        })
    }

    fn oci_dir_root(&self) -> PathBuf {
        self.root.join(".oci-dir")
    }

    /// Add a file to the artifact.
    ///
    /// - The file is compressed by gzip, and added in OCI artifact as a layer.
    /// - Its media type is set as `application/vnd.ocipkg.file+gzip`
    /// - On local storage, the file is stored at the top of image directory.
    pub fn add_file(&mut self, file: &Path) -> Result<()> {
        let bytes: Vec<u8> = todo!();
        let descriptor: Descriptor = todo!();
        self.add_blob(descriptor, &bytes)
    }

    /// Add a directory to the artifact.
    ///
    /// - The directory is archived by tar and compressed by gzip, and then added in OCI artifact as a layer.
    /// - Its media type is set as `application/vnd.ocipkg.directory.tar+gzip`
    /// - On local storage, the directory is stored at the top of image directory.
    pub fn add_directory(&mut self, directory: &Path) -> Result<()> {
        let bytes: Vec<u8> = todo!();
        let descriptor: Descriptor = todo!();
        self.add_blob(descriptor, &bytes)
    }

    /// Add a blob with arbitrary descriptor to the image.
    pub fn add_blob(&mut self, descriptor: Descriptor, blob: &[u8]) -> Result<()> {
        let digest = Digest::new(&descriptor.digest())?;
        // FIXME: check digest of blob
        let path = self.oci_dir_root().join(digest.as_path());
        fs::write(path, blob)?;
        self.manifest.layers_mut().push(descriptor);
        Ok(())
    }

    /// Add a config to the image manifest
    ///
    /// The guideline of OCI artifact has three cases,
    pub fn add_config(&mut self, descriptor: Descriptor, config: &[u8]) -> Result<()> {
        let digest = Digest::new(&descriptor.digest())?;
        // FIXME: check digest of config
        let path = self.oci_dir_root().join(digest.as_path());
        fs::write(path, config)?;
        self.manifest.set_config(descriptor);
        Ok(())
    }

    /// Add an annotation to the image manifest
    pub fn add_annotation(&mut self, key: &str, value: &str) -> Result<()> {
        self.manifest
            .annotations_mut()
            .get_or_insert(Default::default())
            .insert(key.to_string(), value.to_string());
        Ok(())
    }

    /// Set `artifactType` field in the manifest
    pub fn set_artifact_type(&mut self, artifact_type: MediaType) -> Result<()> {
        self.manifest.set_artifact_type(Some(artifact_type));
        Ok(())
    }

    /// Finish writing the image
    pub fn finish(self) -> Result<()> {
        todo!()
    }
}

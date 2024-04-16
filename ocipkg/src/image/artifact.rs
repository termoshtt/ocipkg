use crate::{error::*, local::image_dir, oci_dir::LocalOciDirBuilder, ImageName};
use flate2::{bufread, write, Compression};
use oci_spec::image::{Descriptor, ImageManifest, ImageManifestBuilder, MediaType};
use std::{
    fs,
    io::prelude::*,
    io::BufReader,
    path::{Path, PathBuf},
};

/// Create a new OCI artifact based on [Guidelines for Artifact Usage] of OCI image specification 1.1.0
///
/// The artifact is created on the local storage where [crate::local] manages as a [OCI Image layout].
///
/// [Guidelines for Artifact Usage]: https://github.com/opencontainers/image-spec/blob/v1.1.0/manifest.md#guidelines-for-artifact-usage
/// [OCI Image layout]: https://github.com/opencontainers/image-spec/blob/v1.1.0/image-layout.md
pub struct LocalArtifactBuilder {
    manifest: ImageManifest,
    image_root: PathBuf,
    oci_dir: LocalOciDirBuilder,
}

impl LocalArtifactBuilder {
    pub fn new(image_name: ImageName) -> Result<Self> {
        let image_root = image_dir(&image_name)?;
        if image_root.exists() {
            return Err(Error::ImageAlreadyExists(image_root));
        }
        let oci_dir = LocalOciDirBuilder::new(image_root.to_owned())?;
        Ok(Self {
            image_root,
            manifest: ImageManifestBuilder::default().build()?,
            oci_dir,
        })
    }

    /// Add a file to the artifact.
    ///
    /// - The file is compressed by gzip, and added in OCI artifact as a layer.
    /// - Its media type is set as `application/vnd.ocipkg.file+gzip`
    /// - On local storage, the file is stored at the top of image directory.
    pub fn add_file(&mut self, file: &Path) -> Result<()> {
        if !file.is_file() {
            return Err(Error::NotAFile(file.to_owned()));
        }
        fs::copy(
            file,
            self.image_root
                .join(file.file_name().expect("Already checked")),
        )?;

        let f = fs::File::open(file)?;
        let b = BufReader::new(f);
        let mut gz = bufread::GzEncoder::new(b, Compression::fast());
        let mut bytes = Vec::new();
        gz.read_to_end(&mut bytes)?;

        let digest = self.oci_dir.save_blob(&bytes)?;
        let descriptor = Descriptor::new(
            MediaType::Other("application/vnd.ocipkg.file+gzip".to_string()),
            bytes.len() as i64,
            digest.to_string(),
        );
        self.manifest.layers_mut().push(descriptor);
        Ok(())
    }

    /// Add a directory to the artifact.
    ///
    /// - The directory is archived by tar and compressed by gzip, and then added in OCI artifact as a layer.
    /// - Its media type is set as `application/vnd.ocipkg.directory.tar+gzip`
    /// - On local storage, the directory is stored at the top of image directory.
    pub fn add_directory(&mut self, directory: &Path) -> Result<()> {
        if !directory.is_dir() {
            return Err(Error::NotADirectory(directory.to_owned()));
        }
        fs_extra::dir::copy(
            directory,
            &self.image_root,
            &fs_extra::dir::CopyOptions {
                overwrite: true,
                ..Default::default()
            },
        )?;

        let mut ar = tar::Builder::new(write::GzEncoder::new(Vec::new(), Compression::default()));
        ar.append_dir_all("", directory)?;
        let bytes = ar.into_inner()?.finish()?;

        let digest = self.oci_dir.save_blob(&bytes)?;
        let descriptor = Descriptor::new(
            MediaType::Other("application/vnd.ocipkg.directory.tar+gzip".to_string()),
            bytes.len() as i64,
            digest.to_string(),
        );
        self.manifest.layers_mut().push(descriptor);
        Ok(())
    }

    /// Add a blob with arbitrary descriptor to the image.
    ///
    /// The `size` and `digest` in `descriptor` is overwritten by the actual blob.
    ///
    pub fn add_blob(&mut self, mut descriptor: Descriptor, blob: &[u8]) -> Result<()> {
        let digest = self.oci_dir.save_blob(blob)?;
        descriptor.set_size(blob.len() as i64);
        descriptor.set_digest(digest.to_string());
        self.manifest.layers_mut().push(descriptor);
        Ok(())
    }

    /// Add a config to the image manifest
    ///
    /// Note that OCI artifact can store any type of configuration different from `application/vnd.oci.image.config.v1+json`.
    /// See the third case of [Guidelines for Artifact Usage].
    ///
    /// [Guidelines for Artifact Usage]: https://github.com/opencontainers/image-spec/blob/v1.1.0/manifest.md#guidelines-for-artifact-usage
    pub fn add_config(&mut self, mut descriptor: Descriptor, config: &[u8]) -> Result<()> {
        let digest = self.oci_dir.save_blob(config)?;
        descriptor.set_size(config.len() as i64);
        descriptor.set_digest(digest.to_string());
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
        self.oci_dir.finish(self.manifest)?;
        Ok(())
    }
}

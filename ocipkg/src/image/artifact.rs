//! Compose directory as a container tar

use crate::{
    digest::Digest,
    image::{
        Config, ImageLayout, OciArchive, OciArchiveBuilder, OciArtifact, OciArtifactBuilder, OciDir,
    },
    media_types::{self, config_json},
    ImageName,
};
use anyhow::{bail, Result};
use flate2::{write::GzEncoder, Compression};
use std::{
    collections::HashMap,
    fs,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};

/// Build [Artifact]
pub struct Builder {
    config: Config,
    builder: OciArtifactBuilder<OciArchiveBuilder>,
}

impl Builder {
    pub fn new(path: PathBuf, image_name: ImageName) -> Result<Self> {
        Ok(Builder {
            builder: OciArtifactBuilder::new(
                OciArchiveBuilder::new(path)?,
                media_types::artifact(),
                image_name,
            )?,
            config: Config::default(),
        })
    }

    /// Append a files as a layer
    pub fn append_files(&mut self, ps: &[impl AsRef<Path>]) -> Result<()> {
        let mut ar = tar::Builder::new(GzEncoder::new(Vec::new(), Compression::default()));
        let mut files = Vec::new();
        for path in ps {
            let path = path.as_ref();
            if !path.is_file() {
                bail!("{} is not a file", path.display());
            }
            let name = path
                .file_name()
                .expect("This never fails since checked above")
                .to_str()
                .expect("Non-UTF8 file name");
            let mut f = fs::File::open(path)?;
            files.push(PathBuf::from(name));
            ar.append_file(name, &mut f)?;
        }
        let buf = ar.into_inner()?.finish()?;
        let layer = self
            .builder
            .add_layer(media_types::layer_tar_gzip(), &buf, HashMap::new())?;
        self.config
            .add_layer(Digest::from_descriptor(&layer)?, files);
        Ok(())
    }

    /// Append directory as a layer
    pub fn append_dir_all(&mut self, path: &Path) -> Result<()> {
        if !path.is_dir() {
            bail!("{} is not a directory", path.display());
        }
        let paths = fs::read_dir(path)?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .collect();

        let mut ar = tar::Builder::new(GzEncoder::new(Vec::new(), Compression::default()));
        ar.append_dir_all("", path)?;
        let buf = ar.into_inner()?.finish()?;
        let layer_desc =
            self.builder
                .add_layer(media_types::layer_tar_gzip(), &buf, HashMap::new())?;
        self.config
            .add_layer(Digest::new(layer_desc.digest())?, paths);
        Ok(())
    }

    pub fn build(mut self) -> Result<OciArchive> {
        self.builder.add_config(
            config_json(),
            self.config.to_json()?.as_bytes(),
            HashMap::new(),
        )?;
        self.builder.build()
    }
}

/// ocipkg artifact defined as `application/vnd.ocipkg.v1.artifact`
pub struct Artifact<Base: ImageLayout> {
    base: OciArtifact<Base>,
}

impl<Base: ImageLayout> Deref for Artifact<Base> {
    type Target = Base;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<Base: ImageLayout> DerefMut for Artifact<Base> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl<Base: ImageLayout> Artifact<Base> {
    pub fn new(base: Base) -> Self {
        // TODO: Check media type
        Self {
            base: OciArtifact::new(base),
        }
    }

    /// Get list of files stored in the ocipkg artifact
    pub fn files(&mut self) -> Result<Vec<PathBuf>> {
        todo!()
    }

    /// Unpack ocipkg artifact into local filesystem with `.oci-dir` directory
    pub fn unpack(&mut self, dest: &Path) -> Result<OciDir> {
        todo!()
    }
}

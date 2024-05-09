//! Compose directory as a container tar

use crate::{
    digest::Digest,
    image::{
        copy, Config, Image, OciArchive, OciArchiveBuilder, OciArtifact, OciArtifactBuilder,
        OciDir, OciDirBuilder, Remote,
    },
    local::image_dir,
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
                OciArchiveBuilder::new(path, image_name)?,
                media_types::artifact(),
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

    pub fn build(mut self) -> Result<OciArtifact<OciArchive>> {
        self.builder.add_config(
            config_json(),
            self.config.to_json()?.as_bytes(),
            HashMap::new(),
        )?;
        self.builder.build()
    }
}

/// ocipkg artifact defined as `application/vnd.ocipkg.v1.artifact`
pub struct Artifact<Base: Image> {
    base: OciArtifact<Base>,
}

impl<Base: Image> Deref for Artifact<Base> {
    type Target = OciArtifact<Base>;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<Base: Image> DerefMut for Artifact<Base> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl Artifact<OciArchive> {
    pub fn from_oci_archive(path: &Path) -> Result<Self> {
        let base = OciArtifact::from_oci_archive(path)?;
        Ok(Self { base })
    }
}

impl Artifact<OciDir> {
    pub fn from_oci_dir(path: &Path) -> Result<Self> {
        let base = OciArtifact::from_oci_dir(path)?;
        Ok(Self { base })
    }
}

impl Artifact<Remote> {
    pub fn from_remote(image_name: ImageName) -> Result<Self> {
        let base = OciArtifact::from_remote(image_name)?;
        Ok(Self { base })
    }
}

impl<Base: Image> Artifact<Base> {
    pub fn new(base: Base) -> Result<Self> {
        let mut base = OciArtifact::new(base);
        let ty = base.artifact_type()?;
        if ty != media_types::artifact() {
            bail!("Not an ocipkg artifact: {}", ty);
        }
        Ok(Self { base })
    }

    pub fn get_ocipkg_config(&mut self) -> Result<Config> {
        let (_, buf) = self.base.get_config()?;
        Ok(serde_json::from_slice(&buf)?)
    }

    /// Get list of files stored in the ocipkg artifact
    pub fn files(&mut self) -> Result<Vec<PathBuf>> {
        let config = self.get_ocipkg_config()?;
        Ok(config.layers().values().flatten().cloned().collect())
    }

    /// Unpack ocipkg artifact into local filesystem with `.oci-dir` directory
    pub fn unpack(&mut self, overwrite: bool) -> Result<OciDir> {
        let image_name = self.base.get_name()?;
        let dest = image_dir(&image_name)?;
        if dest.exists() {
            if overwrite {
                log::warn!(
                    "Destination already exists: {}. Removing...",
                    dest.display()
                );
                fs::remove_dir_all(&dest)?;
            } else {
                bail!("Destination already exists: {}", dest.display());
            }
        }
        fs::create_dir_all(&dest)?;
        let oci_dir = OciDirBuilder::new(dest.join(".oci-dir"), self.base.get_name()?)?;
        let oci_dir = copy(self.base.deref_mut(), oci_dir)?;
        for (desc, blob) in self.base.get_layers()? {
            if desc.media_type() == &media_types::layer_tar_gzip() {
                let buf = flate2::read::GzDecoder::new(blob.as_slice());
                tar::Archive::new(buf).unpack(&dest)?;
            } else {
                bail!("Unsupported layer type: {}", desc.media_type());
            }
        }
        Ok(oci_dir)
    }
}

/// Load ocipkg artifact into local storage
pub fn load(input: &Path, overwrite: bool) -> Result<()> {
    let mut ar = Artifact::from_oci_archive(input)?;
    ar.unpack(overwrite)?;
    Ok(())
}

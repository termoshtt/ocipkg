//! Executable container

use super::{OciArchive, OciArchiveBuilder};
use crate::{image::ImageBuilder, ImageName};
use anyhow::Result;
use oci_spec::image::ImageManifestBuilder;
use std::path::PathBuf;

pub struct Runnable {
    archive: OciArchive,
}

pub struct RunnableBuilder {
    builder: OciArchiveBuilder,
    manifest: oci_spec::image::ImageManifest,
}

impl RunnableBuilder {
    pub fn new(path: PathBuf, image_name: ImageName) -> Result<Self> {
        Ok(RunnableBuilder {
            builder: OciArchiveBuilder::new(path, image_name)?,
            manifest: ImageManifestBuilder::default()
                .schema_version(2_u32)
                .build()?,
        })
    }

    pub fn build(self) -> Result<Runnable> {
        Ok(Runnable {
            archive: self.builder.build(self.manifest)?,
        })
    }
}

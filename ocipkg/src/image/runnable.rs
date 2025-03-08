//! Executable container

use std::path::PathBuf;

use super::OciArchiveBuilder;
use crate::{image::ImageBuilder, ImageName};
use anyhow::Result;
use oci_spec::image::{ImageManifest, ImageManifestBuilder};

/// Build [`Runnable`], executable container
pub struct RunnableBuilder<LayoutBuilder: ImageBuilder> {
    manifest: ImageManifest,
    layout: LayoutBuilder,
}

impl<LayoutBuilder: ImageBuilder> RunnableBuilder<LayoutBuilder> {
    pub fn new(builder: LayoutBuilder) -> Result<Self> {
        Ok(Self {
            layout: builder,
            manifest: ImageManifestBuilder::default()
                .schema_version(2_u32)
                .build()?,
        })
    }

    pub fn build(self) -> Result<Runnable<LayoutBuilder::Image>> {
        Ok(Runnable(self.layout.build(self.manifest)?))
    }
}

impl RunnableBuilder<OciArchiveBuilder> {
    pub fn new_archive_unnamed(path: PathBuf) -> Result<Self> {
        let layout = OciArchiveBuilder::new_unnamed(path)?;
        Ok(Self::new(layout)?)
    }

    pub fn new_archive(path: PathBuf, image_name: ImageName) -> Result<Self> {
        let layout = OciArchiveBuilder::new(path, image_name)?;
        Ok(Self::new(layout)?)
    }
}

pub struct Runnable<Layout>(Layout);

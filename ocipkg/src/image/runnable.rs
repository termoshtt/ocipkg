//! Executable container

use std::path::PathBuf;

use super::OciArchiveBuilder;
use crate::{image::ImageBuilder, ImageName};
use anyhow::Result;
use oci_spec::image::{ConfigBuilder, DescriptorBuilder, ImageManifestBuilder};

/// Build [`Runnable`], executable container
pub struct RunnableBuilder<LayoutBuilder: ImageBuilder> {
    manifest: ImageManifestBuilder,
    config: ConfigBuilder,
    layout: LayoutBuilder,
}

impl<LayoutBuilder: ImageBuilder> RunnableBuilder<LayoutBuilder> {
    pub fn new(builder: LayoutBuilder) -> Result<Self> {
        Ok(Self {
            layout: builder,
            manifest: ImageManifestBuilder::default().schema_version(2_u32),
            config: ConfigBuilder::default(),
        })
    }

    pub fn append_executable(&mut self, path: &PathBuf) -> Result<()> {
        // FIXME
        dbg!(path);

        Ok(())
    }

    pub fn build(mut self) -> Result<Runnable<LayoutBuilder::Image>> {
        let config = self.config.build()?;
        let cfg_json = serde_json::to_string(&config)?;
        let (digest, size) = self.layout.add_blob(cfg_json.as_bytes())?;
        let cfg_desc = DescriptorBuilder::default()
            .media_type(oci_spec::image::MediaType::ImageConfig)
            .size(size)
            .digest(digest)
            .build()?;

        // FIXME
        let layers = Vec::new();

        let manifest = self.manifest.config(cfg_desc).layers(layers).build()?;
        Ok(Runnable(self.layout.build(manifest)?))
    }
}

impl RunnableBuilder<OciArchiveBuilder> {
    pub fn new_archive_unnamed(path: PathBuf) -> Result<Self> {
        let layout = OciArchiveBuilder::new_unnamed(path)?;
        Self::new(layout)
    }

    pub fn new_archive(path: PathBuf, image_name: ImageName) -> Result<Self> {
        let layout = OciArchiveBuilder::new(path, image_name)?;
        Self::new(layout)
    }
}

/// Runnable container containing single, statically linked executable
pub struct Runnable<Layout>(Layout);

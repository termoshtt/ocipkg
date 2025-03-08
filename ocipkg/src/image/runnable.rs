//! Executable container

use std::path::PathBuf;

use super::OciArchiveBuilder;
use crate::{image::ImageBuilder, ImageName};
use anyhow::{ensure, Result};
use oci_spec::image::{
    ConfigBuilder, DescriptorBuilder, ImageConfigurationBuilder, ImageManifestBuilder,
};

/// Build [`Runnable`], executable container
pub struct RunnableBuilder<LayoutBuilder: ImageBuilder> {
    manifest: ImageManifestBuilder,
    entrypoint: Vec<String>,
    layout: LayoutBuilder,
    layers: Vec<oci_spec::image::Descriptor>,
}

impl<LayoutBuilder: ImageBuilder> RunnableBuilder<LayoutBuilder> {
    pub fn new(builder: LayoutBuilder) -> Result<Self> {
        Ok(Self {
            layout: builder,
            manifest: ImageManifestBuilder::default().schema_version(2_u32),
            entrypoint: Vec::new(),
            layers: Vec::new(),
        })
    }

    pub fn append_executable(&mut self, path: &PathBuf) -> Result<()> {
        if !path.is_file() {
            anyhow::bail!("File does not exist: {:?}", path);
        }
        if !self.layers.is_empty() {
            anyhow::bail!("Only one executable is allowed");
        }

        let filename = path
            .file_name()
            .unwrap() // Checked above
            .to_str()
            .expect("Non-UTF8 filename");

        let mut buf = Vec::new();
        {
            let mut tar_builder = tar::Builder::new(&mut buf);
            let mut file = std::fs::File::open(path)?;
            tar_builder.append_file(filename, &mut file)?;
            log::info!("Put executable `/{}` into container", filename);
        }

        let (digest, size) = self.layout.add_blob(&buf)?;

        let layer_desc = DescriptorBuilder::default()
            .media_type(oci_spec::image::MediaType::ImageLayer)
            .size(size)
            .digest(digest)
            .build()?;
        self.layers.push(layer_desc);

        self.entrypoint.push(format!("/{filename}"));

        Ok(())
    }

    pub fn build(mut self) -> Result<Runnable<LayoutBuilder::Image>> {
        ensure!(
            !self.layers.is_empty() && !self.entrypoint.is_empty(),
            "No executable provided. Use `append_executable` to add one"
        );

        let cfg = ImageConfigurationBuilder::default()
            .config(
                ConfigBuilder::default()
                    .entrypoint(self.entrypoint)
                    .working_dir("/")
                    .build()?,
            )
            .build()?;
        let (digest, size) = self
            .layout
            .add_blob(serde_json::to_string(&cfg)?.as_bytes())?;
        let cfg_desc = DescriptorBuilder::default()
            .media_type(oci_spec::image::MediaType::ImageConfig)
            .size(size)
            .digest(digest)
            .build()?;

        let manifest = self.manifest.config(cfg_desc).layers(self.layers).build()?;
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

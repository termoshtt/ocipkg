use crate::{
    image::{ImageLayout, ImageLayoutBuilder},
    Digest, ImageName,
};
use anyhow::{Context, Result};
use oci_spec::image::{
    Descriptor, DescriptorBuilder, ImageManifest, ImageManifestBuilder, MediaType,
};
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

/// Build a [OciArtifact]
pub struct OciArtifactBuilder<LayoutBuilder: ImageLayoutBuilder> {
    name: ImageName,
    manifest: ImageManifest,
    layout: LayoutBuilder,
}

impl<LayoutBuilder: ImageLayoutBuilder> OciArtifactBuilder<LayoutBuilder> {
    /// Create a new OCI Artifact with its media type
    pub fn new(
        mut layout: LayoutBuilder,
        artifact_type: MediaType,
        name: ImageName,
    ) -> Result<Self> {
        let empty_config = layout.add_empty_json()?;
        let manifest = ImageManifestBuilder::default()
            .schema_version(2_u32)
            .artifact_type(artifact_type)
            .config(empty_config)
            .layers(Vec::new())
            .build()?;
        Ok(Self {
            layout,
            manifest,
            name,
        })
    }

    /// Add `config` of the OCI Artifact
    ///
    /// Image manifest of artifact can store any type of configuration blob.
    pub fn add_config(
        &mut self,
        config_type: MediaType,
        config_blob: &[u8],
        annotations: HashMap<String, String>,
    ) -> Result<Descriptor> {
        let (digest, size) = self.layout.add_blob(config_blob)?;
        let config = DescriptorBuilder::default()
            .media_type(config_type)
            .annotations(annotations)
            .digest(digest.to_string())
            .size(size)
            .build()?;
        self.manifest.set_config(config.clone());
        Ok(config)
    }

    /// Append a `layer` to the OCI Artifact
    ///
    /// Image manifest of artifact can store any type of layer blob.
    pub fn add_layer(
        &mut self,
        layer_type: MediaType,
        layer_blob: &[u8],
        annotations: HashMap<String, String>,
    ) -> Result<Descriptor> {
        let (digest, size) = self.layout.add_blob(layer_blob)?;
        let layer = DescriptorBuilder::default()
            .media_type(layer_type)
            .digest(digest.to_string())
            .size(size)
            .annotations(annotations)
            .build()?;
        self.manifest.layers_mut().push(layer.clone());
        Ok(layer)
    }

    /// Build the OCI Artifact
    pub fn build(self) -> Result<OciArtifact<LayoutBuilder::ImageLayout>> {
        Ok(OciArtifact::new(
            self.layout.build(self.manifest, self.name)?,
        ))
    }
}

/// OCI Artifact, an image layout with a image manifest which stores any type of `config` and `layers` rather than runnable container.
///
/// This is a thin wrapper of an actual image layout implementing [ImageLayout] to provide a common interface for OCI Artifacts.
pub struct OciArtifact<Layout: ImageLayout>(Layout);

impl<Base: ImageLayout> Deref for OciArtifact<Base> {
    type Target = Base;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Layout: ImageLayout> DerefMut for OciArtifact<Layout> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<Layout: ImageLayout> OciArtifact<Layout> {
    pub fn new(layout: Layout) -> Self {
        Self(layout)
    }

    pub fn artifact_type(&mut self) -> Result<MediaType> {
        let (_image_name, manifest) = self.get_manifest()?;
        manifest
            .artifact_type()
            .clone()
            .context("artifactType is not specified in manifest")
    }

    pub fn get_config(&mut self) -> Result<(Descriptor, Vec<u8>)> {
        let (_image_name, manifest) = self.get_manifest()?;
        let config_desc = manifest.config();
        if config_desc.media_type() == &MediaType::EmptyJSON {
            return Ok((config_desc.clone(), "{}".as_bytes().to_vec()));
        }
        let blob = self.get_blob(&Digest::from_descriptor(&config_desc)?)?;
        Ok((config_desc.clone(), blob))
    }

    pub fn get_layers(&mut self) -> Result<Vec<(Descriptor, Vec<u8>)>> {
        let (_image_name, manifest) = self.get_manifest()?;
        manifest
            .layers()
            .iter()
            .map(|layer| {
                let blob = self.get_blob(&Digest::from_descriptor(&layer)?)?;
                Ok((layer.clone(), blob))
            })
            .collect()
    }
}

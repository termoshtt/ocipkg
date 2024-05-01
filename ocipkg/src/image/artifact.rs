use crate::{image::ImageLayoutBuilder, ImageName};
use anyhow::Result;
use oci_spec::image::{
    Descriptor, DescriptorBuilder, ImageManifest, ImageManifestBuilder, MediaType,
};
use std::collections::HashMap;

/// Create a new OCI Artifact over [ImageLayoutBuilder]
///
/// This creates a generic OCI Artifact, not the ocipkg artifact defined as `application/vnd.ocipkg.v1.artifact`.
/// It is the task of the [crate::image::Builder].
pub struct ArtifactBuilder<Base: ImageLayoutBuilder> {
    name: ImageName,
    manifest: ImageManifest,
    layout: Base,
}

impl<Base: ImageLayoutBuilder> ArtifactBuilder<Base> {
    /// Create a new OCI Artifact with its media type
    pub fn new(mut layout: Base, artifact_type: MediaType, name: ImageName) -> Result<Self> {
        // OCI Artifact allows empty JSON blob as a configuration. This is a placeholder for it.
        let empty_config = DescriptorBuilder::default()
            .media_type(MediaType::EmptyJSON)
            .digest(layout.add_empty_json_blob()?.to_string())
            .build()?;
        let manifest = ImageManifestBuilder::default()
            .artifact_type(artifact_type)
            .config(empty_config)
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
        let config = DescriptorBuilder::default()
            .media_type(config_type)
            .annotations(annotations)
            .digest(self.layout.add_blob(config_blob)?.to_string())
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
        let layer = DescriptorBuilder::default()
            .media_type(layer_type)
            .digest(self.layout.add_blob(layer_blob)?.to_string())
            .annotations(annotations)
            .build()?;
        self.manifest.layers_mut().push(layer.clone());
        Ok(layer)
    }

    /// Build the OCI Artifact
    pub fn build(self) -> Result<Base::ImageLayout> {
        self.layout.build(self.manifest, self.name)
    }
}

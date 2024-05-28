use crate::{
    image::{Image, ImageBuilder, OciArchive, OciDir, Remote},
    Digest, ImageName,
};
use anyhow::{Context, Result};
use chrono::{DateTime, TimeZone};
use oci_spec::image::{
    Descriptor, DescriptorBuilder, ImageManifest, ImageManifestBuilder, MediaType,
};
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    path::Path,
};
use url::Url;

/// Build a [OciArtifact]
pub struct OciArtifactBuilder<LayoutBuilder: ImageBuilder> {
    manifest: ImageManifest,
    layout: LayoutBuilder,
}

impl<LayoutBuilder: ImageBuilder> OciArtifactBuilder<LayoutBuilder> {
    /// Create a new OCI Artifact with its media type
    pub fn new(mut layout: LayoutBuilder, artifact_type: MediaType) -> Result<Self> {
        let empty_config = layout.add_empty_json()?;
        let manifest = ImageManifestBuilder::default()
            .schema_version(2_u32)
            .artifact_type(artifact_type)
            .config(empty_config)
            .layers(Vec::new())
            .build()?;
        Ok(Self { layout, manifest })
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

    /// Add any type of annotation to the manifest of the OCI Artifact
    pub fn add_annotation(&mut self, key: String, value: String) {
        self.manifest
            .annotations_mut()
            .get_or_insert(HashMap::new())
            .insert(key, value);
    }

    /// Set `org.opencontainers.image.description` annotation
    ///
    /// Note that ghcr.io [requires](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry) the length of this description in 512 characters or less.
    /// But it is not enforced by the OCI specification, and the length of the description is not limited here.
    pub fn add_description(&mut self, description: String) {
        self.add_annotation(
            "org.opencontainers.image.description".to_string(),
            description,
        )
    }

    /// Set `org.opencontainers.image.source` annotation which helps to track the source code of the image
    ///
    /// Note that ghcr.io [uses](https://docs.github.com/en/packages/learn-github-packages/connecting-a-repository-to-a-package#connecting-a-repository-to-a-container-image-using-the-command-line) this annotation to connect container image to source code repository.
    pub fn add_source(&mut self, url: &Url) {
        self.add_annotation(
            "org.opencontainers.image.source".to_string(),
            url.to_string(),
        )
    }

    /// Set `org.opencontainers.image.url` annotation, URL to find more information on the image
    pub fn add_url(&mut self, url: &Url) {
        self.add_annotation("org.opencontainers.image.url".to_string(), url.to_string())
    }

    /// Set `org.opencontainers.image.created` annotation, date and time on which the image was built.
    pub fn add_created<TZ: TimeZone>(&mut self, created: &DateTime<TZ>) {
        self.add_annotation(
            "org.opencontainers.image.created".to_string(),
            created.to_rfc3339(),
        )
    }

    /// Build the OCI Artifact
    pub fn build(self) -> Result<OciArtifact<LayoutBuilder::Image>> {
        Ok(OciArtifact::new(self.layout.build(self.manifest)?))
    }
}

/// OCI Artifact, an image layout with a image manifest which stores any type of `config` and `layers` rather than runnable container.
///
/// This is a thin wrapper of an actual image layout implementing [Image] to provide a common interface for OCI Artifacts.
pub struct OciArtifact<Layout: Image>(Layout);

impl<Base: Image> Deref for OciArtifact<Base> {
    type Target = Base;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Layout: Image> DerefMut for OciArtifact<Layout> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl OciArtifact<OciArchive> {
    pub fn from_oci_archive(path: &Path) -> Result<Self> {
        let layout = OciArchive::new(path)?;
        Ok(Self(layout))
    }
}

impl OciArtifact<OciDir> {
    pub fn from_oci_dir(path: &Path) -> Result<Self> {
        let layout = OciDir::new(path)?;
        Ok(Self(layout))
    }
}

impl OciArtifact<Remote> {
    pub fn from_remote(image_name: ImageName) -> Result<Self> {
        let layout = Remote::new(image_name)?;
        Ok(Self(layout))
    }
}

impl<Layout: Image> OciArtifact<Layout> {
    pub fn new(layout: Layout) -> Self {
        Self(layout)
    }

    pub fn artifact_type(&mut self) -> Result<MediaType> {
        let manifest = self.get_manifest()?;
        manifest
            .artifact_type()
            .clone()
            .context("artifactType is not specified in manifest")
    }

    pub fn get_config(&mut self) -> Result<(Descriptor, Vec<u8>)> {
        let manifest = self.get_manifest()?;
        let config_desc = manifest.config();
        if config_desc.media_type() == &MediaType::EmptyJSON {
            return Ok((config_desc.clone(), "{}".as_bytes().to_vec()));
        }
        let blob = self.get_blob(&Digest::from_descriptor(config_desc)?)?;
        Ok((config_desc.clone(), blob))
    }

    pub fn get_layers(&mut self) -> Result<Vec<(Descriptor, Vec<u8>)>> {
        let manifest = self.get_manifest()?;
        manifest
            .layers()
            .iter()
            .map(|layer| {
                let blob = self.get_blob(&Digest::from_descriptor(layer)?)?;
                Ok((layer.clone(), blob))
            })
            .collect()
    }
}

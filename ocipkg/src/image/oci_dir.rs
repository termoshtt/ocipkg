use crate::{
    image::{Image, ImageBuilder},
    Digest, ImageName,
};
use anyhow::{bail, Context, Result};
use maplit::hashmap;
use oci_spec::image::{
    DescriptorBuilder, ImageIndex, ImageIndexBuilder, ImageManifest, MediaType, OciLayout,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

use super::get_name_from_index;

/// Build an [OciDir]
pub struct OciDirBuilder {
    image_name: Option<ImageName>,
    oci_dir_root: PathBuf,
    is_finished: bool,
}

impl Drop for OciDirBuilder {
    fn drop(&mut self) {
        // Remove oci-dir if it is not finished.
        if !self.is_finished {
            fs::remove_dir_all(&self.oci_dir_root).unwrap_or_else(|e| {
                log::error!(
                    "Failed to remove oci-dir {}: {}",
                    self.oci_dir_root.display(),
                    e
                )
            });
        }
    }
}

impl OciDirBuilder {
    pub fn new_unnamed(oci_dir_root: PathBuf) -> Result<Self> {
        if oci_dir_root.exists() {
            bail!("oci-dir {} already exists", oci_dir_root.display());
        }
        fs::create_dir_all(&oci_dir_root)?;
        Ok(Self {
            image_name: None,
            oci_dir_root,
            is_finished: false,
        })
    }

    pub fn new(oci_dir_root: PathBuf, image_name: ImageName) -> Result<Self> {
        if oci_dir_root.exists() {
            bail!("oci-dir {} already exists", oci_dir_root.display());
        }
        fs::create_dir_all(&oci_dir_root)?;
        Ok(Self {
            image_name: Some(image_name),
            oci_dir_root,
            is_finished: false,
        })
    }
}

impl ImageBuilder for OciDirBuilder {
    type Image = OciDir;

    fn add_blob(&mut self, data: &[u8]) -> Result<(Digest, i64)> {
        let digest = Digest::from_buf_sha256(data);
        let out = self.oci_dir_root.join(digest.as_path());
        fs::create_dir_all(out.parent().unwrap())?;
        fs::write(out, data)?;
        Ok((digest, data.len() as i64))
    }

    fn build(mut self, manifest: ImageManifest) -> Result<OciDir> {
        let manifest_json = serde_json::to_string(&manifest)?;
        let (digest, size) = self.add_blob(manifest_json.as_bytes())?;
        let descriptor = DescriptorBuilder::default()
            .media_type(MediaType::ImageManifest)
            .size(size)
            .digest(digest.to_string())
            .annotations(if let Some(name) = &self.image_name {
                hashmap! {
                    "org.opencontainers.image.ref.name".to_string() => name.to_string()
                }
            } else {
                hashmap! {}
            })
            .build()?;
        let index = ImageIndexBuilder::default()
            .schema_version(2_u32)
            .manifests(vec![descriptor])
            .build()?;
        fs::write(
            self.oci_dir_root.join("oci-layout"),
            r#"{"imageLayoutVersion":"1.0.0"}"#,
        )?;
        fs::write(
            self.oci_dir_root.join("index.json"),
            serde_json::to_string(&index)?,
        )?;
        self.is_finished = true;
        Ok(OciDir {
            oci_dir_root: self.oci_dir_root.clone(),
        })
    }
}

/// `oci-dir` image layout, a directory in the form of [OCI Image Layout](https://github.com/opencontainers/image-spec/blob/v1.1.0/image-layout.md).
///
/// The name "oci-dir" comes from [`podman save`](https://docs.podman.io/en/latest/markdown/podman-save.1.html).
pub struct OciDir {
    oci_dir_root: PathBuf,
}

impl OciDir {
    pub fn new(oci_dir_root: &Path) -> Result<Self> {
        if !oci_dir_root.is_dir() {
            bail!("{} is not a directory", oci_dir_root.display());
        }
        let oci_layout: OciLayout = fs::read(oci_dir_root.join("oci-layout"))
            .and_then(|bytes| Ok(serde_json::from_slice(&bytes)?))
            .context("The directory is not a oci-dir; oci-layout is not found.")?;
        if oci_layout.image_layout_version() != "1.0.0" {
            bail!(
                "Incompatible oci-layout version in {}",
                oci_dir_root.display()
            );
        }
        Ok(Self {
            oci_dir_root: oci_dir_root.to_owned(),
        })
    }

    fn get_index(&mut self) -> Result<ImageIndex> {
        let index_path = self.oci_dir_root.join("index.json");
        let index_json = fs::read_to_string(index_path)?;
        Ok(serde_json::from_str(&index_json)?)
    }
}

impl Image for OciDir {
    fn get_name(&mut self) -> Result<ImageName> {
        get_name_from_index(&self.get_index()?)
    }

    fn get_blob(&mut self, digest: &Digest) -> Result<Vec<u8>> {
        Ok(fs::read(self.oci_dir_root.join(digest.as_path()))?)
    }

    fn get_manifest(&mut self) -> Result<ImageManifest> {
        let index = self.get_index()?;
        let desc = index
            .manifests()
            .first()
            .context("No manifest found in index.json")?;
        let digest = Digest::from_descriptor(desc)?;
        let manifest = serde_json::from_slice(self.get_blob(&digest)?.as_slice())?;
        Ok(manifest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::OciArtifactBuilder;

    #[test]
    fn test_artifact_over_oci_dir() -> Result<()> {
        let tmp_dir = tempfile::tempdir()?;
        let path = tmp_dir.path().join("oci-dir");
        let image_name = ImageName::parse("test")?;
        let oci_dir = OciDirBuilder::new(path, image_name.clone())?;
        let mut artifact =
            OciArtifactBuilder::new(oci_dir, MediaType::Other("test".to_string()))?.build()?;

        let name = artifact.get_name()?;
        let manifest = artifact.get_manifest()?;
        assert_eq!(name, image_name);
        assert_eq!(
            manifest.artifact_type().as_ref().unwrap(),
            &MediaType::Other("test".to_string())
        );

        let (config_desc, config) = artifact.get_config()?;
        assert_eq!(config_desc.media_type(), &MediaType::EmptyJSON);
        assert_eq!(config, "{}".as_bytes());

        Ok(())
    }
}

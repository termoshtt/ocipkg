use crate::{
    image::{ImageLayout, ImageLayoutBuilder},
    Digest, ImageName,
};
use anyhow::{bail, Context, Result};
use maplit::hashmap;
use oci_spec::image::{
    DescriptorBuilder, ImageIndex, ImageIndexBuilder, ImageManifest, MediaType, OciLayout,
};
use std::{fs, path::PathBuf};

/// Build an [OciDir]
pub struct OciDirBuilder {
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
    pub fn new(oci_dir_root: PathBuf) -> Result<Self> {
        if oci_dir_root.exists() {
            bail!("oci-dir {} already exists", oci_dir_root.display());
        }
        fs::create_dir_all(&oci_dir_root)?;
        Ok(Self {
            oci_dir_root,
            is_finished: false,
        })
    }
}

impl ImageLayoutBuilder for OciDirBuilder {
    type ImageLayout = OciDir;

    fn add_blob(&mut self, data: &[u8]) -> Result<(Digest, i64)> {
        let digest = Digest::from_buf_sha256(data);
        let out = self.oci_dir_root.join(digest.as_path());
        fs::create_dir_all(out.parent().unwrap())?;
        fs::write(out, data)?;
        Ok((digest, data.len() as i64))
    }

    fn build(mut self, manifest: ImageManifest, image_name: ImageName) -> Result<OciDir> {
        let manifest_json = serde_json::to_string(&manifest)?;
        let (digest, size) = self.add_blob(manifest_json.as_bytes())?;
        let descriptor = DescriptorBuilder::default()
            .media_type(MediaType::ImageManifest)
            .size(size)
            .digest(digest.to_string())
            .annotations(hashmap! {
                "org.opencontainers.image.ref.name".to_string() => image_name.to_string(),
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
    pub fn new(oci_dir_root: PathBuf) -> Result<Self> {
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
        Ok(Self { oci_dir_root })
    }
}

impl ImageLayout for OciDir {
    fn get_index(&mut self) -> Result<ImageIndex> {
        let index_path = self.oci_dir_root.join("index.json");
        let index_json = fs::read_to_string(index_path)?;
        Ok(serde_json::from_str(&index_json)?)
    }

    fn get_blob(&mut self, digest: &Digest) -> Result<Vec<u8>> {
        Ok(fs::read(self.oci_dir_root.join(digest.as_path()))?)
    }
}

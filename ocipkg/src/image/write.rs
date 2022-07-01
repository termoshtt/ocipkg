//! Compose directory as a container tar

use anyhow::{bail, Context};
use flate2::{write::GzEncoder, Compression};
use oci_spec::image::*;
use std::{convert::TryFrom, io, path::Path, time::SystemTime};

use crate::Digest;

/// Build a container in oci-archive format based
/// on the [OCI image spec](https://github.com/opencontainers/image-spec)
pub struct Builder<W: io::Write> {
    builder: tar::Builder<W>,
    config: Option<Descriptor>,
    layers: Vec<Descriptor>,
}

impl<W: io::Write> Builder<W> {
    pub fn new(writer: W) -> Self {
        Builder {
            builder: tar::Builder::new(writer),
            config: None,
            layers: Vec::new(),
        }
    }

    pub fn append_config(&mut self, cfg: ImageConfiguration) -> anyhow::Result<()> {
        let mut buf = Vec::new();
        cfg.to_writer(&mut buf)?;
        let config_desc = save_blob(&mut self.builder, MediaType::ImageConfig, &buf)?;
        if self.config.replace(config_desc).is_some() {
            bail!("ImageConfiguration is set twice.")
        } else {
            Ok(())
        }
    }

    /// Append a file as a signle-file layer
    pub fn append_file(&mut self, _path: &Path) -> anyhow::Result<()> {
        todo!()
    }

    /// Append directory as a layer
    pub fn append_dir_all(&mut self, path: &Path) -> anyhow::Result<()> {
        let buf = create_tar_gz_on_memory_from_dir(path, "rootfs-c9d-v1")?;
        let layer_desc = save_blob(&mut self.builder, MediaType::ImageLayerGzip, &buf)?;
        self.layers.push(layer_desc);
        Ok(())
    }

    pub fn into_inner(mut self) -> anyhow::Result<W> {
        let image_manifest = ImageManifestBuilder::default()
            .schema_version(SCHEMA_VERSION)
            .config(self.config.context("ImageConfiguration is not set")?)
            .layers(self.layers.clone())
            .build()?;
        let mut buf = Vec::new();
        image_manifest.to_writer(&mut buf)?;
        let image_manifest_desc = save_blob(&mut self.builder, MediaType::ImageManifest, &buf)?;

        let index = ImageIndexBuilder::default()
            .schema_version(SCHEMA_VERSION)
            .manifests(vec![image_manifest_desc])
            .build()?;
        let mut index_json = Vec::new();
        index.to_writer(&mut index_json)?;
        let index_json = String::from_utf8(index_json)?;
        save_file(&mut self.builder, Path::new("index.json"), &index_json)?;

        let version = r#"{"imageLayoutVersion":"1.0.0"}"#;
        save_file(&mut self.builder, Path::new("oci-layout"), version)?;

        Ok(self.builder.into_inner()?)
    }
}

fn now_mtime() -> u64 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}

fn create_header(size: usize) -> tar::Header {
    let mut header = tar::Header::new_gnu();
    header.set_size(u64::try_from(size).unwrap());
    header.set_cksum();
    header.set_mode(0b110100100); // rw-r--r--
    header.set_mtime(now_mtime());
    header
}

fn save_blob<W: io::Write>(
    builder: &mut tar::Builder<W>,
    media_type: MediaType,
    buf: &[u8],
) -> anyhow::Result<Descriptor> {
    let digest = Digest::from_buf_sha256(buf);

    let mut header = create_header(buf.len());
    builder
        .append_data(&mut header, digest.as_path(), buf)
        .context("IO error while writing tar achive")?;

    Ok(DescriptorBuilder::default()
        .media_type(media_type)
        .size(i64::try_from(buf.len())?)
        .digest(format!("{}", digest))
        .build()
        .expect("Requirement for descriptor is mediaType, digest, and size."))
}

fn save_file<W: io::Write>(
    builder: &mut tar::Builder<W>,
    dest: &Path,
    input: &str,
) -> anyhow::Result<()> {
    let mut header = create_header(input.len());
    builder
        .append_data(&mut header, dest, input.as_bytes())
        .context("IO error while writing tar achive")?;
    Ok(())
}

/// Compose input directory as a tar.gz archive on memory
fn create_tar_gz_on_memory_from_dir(input: &Path, rootfs_name: &str) -> anyhow::Result<Vec<u8>> {
    let encoder = GzEncoder::new(Vec::new(), Compression::default());
    let mut ar = tar::Builder::new(encoder);
    ar.append_dir_all(rootfs_name, &input)
        .context("Error while reading input directory")?;
    Ok(ar
        .into_inner()
        .expect("This never fails since tar arhive is creating on memory")
        .finish()
        .expect("This never fails since zip is creating on memory"))
}

/// Compose a directory as container in oci-archive format based
/// on the [OCI image spec](https://github.com/opencontainers/image-spec)
pub fn pack_dir<W: io::Write>(input_directory: &Path, output: W) -> anyhow::Result<()> {
    if !input_directory.is_dir() {
        bail!(
            "Input directory is not a directory: {}",
            input_directory.display()
        );
    }

    let mut b = Builder::new(output);
    b.append_config(ImageConfigurationBuilder::default().build()?)?;
    b.append_dir_all(input_directory)?;
    let _output = b.into_inner()?;

    Ok(())
}

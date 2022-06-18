//! Compose directory as a container tar

use anyhow::{bail, Context};
use flate2::{write::GzEncoder, Compression};
use oci_spec::image::*;
use sha2::{Digest, Sha256};
use std::{convert::TryFrom, fs, io, path::Path, time::SystemTime};

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
    let hash = Sha256::digest(&buf);
    let digest = base16ct::lower::encode_string(&hash);

    let mut header = create_header(buf.len());
    builder
        .append_data(&mut header, format!("blobs/sha256/{}", digest), buf)
        .context("IO error while writing tar achive")?;

    Ok(DescriptorBuilder::default()
        .media_type(media_type)
        .size(i64::try_from(buf.len())?)
        .digest(format!("sha256:{}", digest))
        .build()
        .expect("Requirement for descriptor is mediaType, digest, and size."))
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

/// Compose a directory as container in oci-archive format based on the [OCI image spec](https://github.com/opencontainers/image-spec)
pub fn compose(input_directory: &Path, output: &Path) -> anyhow::Result<()> {
    if !input_directory.is_dir() {
        panic!(
            "Input directory is not a directory: {}",
            input_directory
                .to_str()
                .expect("Non-UTF8 input is not supported")
        );
    }
    let mut output = output.to_owned();
    output.set_extension("tar");
    if output.exists() {
        bail!("Output directory already exists");
    }

    let mut oci_archive = tar::Builder::new(fs::File::create(output)?);

    let buf = create_tar_gz_on_memory_from_dir(&input_directory, "rootfs-c9d-v1")?;
    let layer_desc = save_blob(&mut oci_archive, MediaType::ImageLayerGzip, &buf)?;

    // No configuration
    let cfg = ImageConfigurationBuilder::default().build()?;
    let mut buf = Vec::new();
    cfg.to_writer(&mut buf)?;
    let config_desc = save_blob(&mut oci_archive, MediaType::ImageConfig, &buf)?;

    let image_manifest = ImageManifestBuilder::default()
        .schema_version(SCHEMA_VERSION)
        .config(config_desc)
        .layers(vec![layer_desc])
        .build()?;
    let mut buf = Vec::new();
    image_manifest.to_writer(&mut buf)?;
    let image_manifest_desc = save_blob(&mut oci_archive, MediaType::ImageManifest, &buf)?;

    let index = ImageIndexBuilder::default()
        .schema_version(SCHEMA_VERSION)
        .manifests(vec![image_manifest_desc])
        .build()?;
    let mut index_json = Vec::<u8>::new();
    index.to_writer(&mut index_json)?;
    oci_archive.append_data(
        &mut create_header(index_json.len()),
        "index.json",
        index_json.as_slice(),
    )?;

    let version = r#"{"imageLayoutVersion":"1.0.0"}"#;
    oci_archive.append_data(
        &mut create_header(version.len()),
        "oci-layout",
        version.as_bytes(),
    )?;

    Ok(())
}

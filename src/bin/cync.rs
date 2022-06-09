use anyhow::bail;
use flate2::{write::GzEncoder, Compression};
use oci_spec::image::*;
use sha2::{Digest, Sha256};
use std::{
    convert::TryFrom,
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "cync",
    about = "Container for binary distribution without virtualization"
)]
struct Opt {
    /// Input directory
    #[structopt(parse(from_os_str))]
    input_directory: PathBuf,

    /// Output oci archive directory
    #[structopt(parse(from_os_str))]
    output: PathBuf,
}

fn save_blob(blob_root: &Path, media_type: MediaType, buf: &[u8]) -> anyhow::Result<Descriptor> {
    let hash = Sha256::digest(&buf);
    let digest = base16ct::lower::encode_string(&hash);
    let mut out = fs::File::create(blob_root.join(&digest))?;
    out.write_all(&buf)?;
    Ok(DescriptorBuilder::default()
        .media_type(media_type)
        .size(i64::try_from(buf.len())?)
        .digest(format!("sha256:{}", digest))
        .build()?)
}

fn main() -> anyhow::Result<()> {
    let Opt {
        input_directory,
        output,
    } = Opt::from_args();
    if !input_directory.is_dir() {
        panic!(
            "Input directory is not a directory: {}",
            input_directory
                .to_str()
                .expect("Non-UTF8 input is not supported")
        );
    }
    if output.exists() {
        bail!("Output directory already exists");
    }

    let blob_root = output.join("blobs").join("sha256");
    fs::create_dir_all(&blob_root)?;

    // Compose input directory as a tar.gz archive
    let encoder = GzEncoder::new(Vec::new(), Compression::default());
    let mut ar = tar::Builder::new(encoder);
    ar.append_dir_all("rootfs-c9d-v1", &input_directory)?;
    let buf: Vec<u8> = ar.into_inner()?.finish()?;
    let layer_desc = save_blob(&blob_root, MediaType::ImageLayerGzip, &buf)?;

    let cfg = ImageConfigurationBuilder::default().build()?;
    let mut buf = Vec::new();
    cfg.to_writer(&mut buf)?;
    let config_desc = save_blob(&blob_root, MediaType::ImageConfig, &buf)?;

    let image_manifest = ImageManifestBuilder::default()
        .schema_version(SCHEMA_VERSION)
        .config(config_desc)
        .layers(vec![layer_desc])
        .build()?;
    let mut buf = Vec::new();
    image_manifest.to_writer(&mut buf)?;
    let image_manifest_desc = save_blob(&blob_root, MediaType::ImageManifest, &buf)?;
    dbg!(image_manifest_desc);

    Ok(())
}

use anyhow::bail;
use oci_spec::image::{
    Descriptor, DescriptorBuilder, ImageManifestBuilder, MediaType, SCHEMA_VERSION,
};
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

fn calc_digest_sha256(buf: &[u8]) -> String {
    let hash = Sha256::digest(&buf);
    base16ct::lower::encode_string(&hash)
}

fn save_blob(blob_root: &Path, media_type: MediaType, buf: &[u8]) -> anyhow::Result<Descriptor> {
    let digest = calc_digest_sha256(&buf);
    let mut out = fs::File::create(blob_root.join("sha256").join(&digest))?;
    out.write_all(&buf)?;
    Ok(DescriptorBuilder::default()
        .media_type(media_type)
        .size(i64::try_from(buf.len())?)
        .digest(digest)
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

    let blob_root = output.join("blobs");
    fs::create_dir_all(&blob_root)?;

    let config = DescriptorBuilder::default()
        .media_type(MediaType::ImageConfig)
        .size(7023)
        .digest("sha256:b5b2b2c507a0944348e0303114d8d93aaaa081732b86451d9bce1f432a537bc7")
        .build()?;

    // Create container as oci-dir format
    let mut ar = tar::Builder::new(Vec::new());
    ar.append_dir_all("rootfs-c9d-v1", &input_directory)?;
    let buf: Vec<u8> = ar.into_inner()?;

    let desc = save_blob(&blob_root, MediaType::ImageLayer, &buf)?;
    let layers: Vec<Descriptor> = vec![desc];

    let image_manifest = ImageManifestBuilder::default()
        .schema_version(SCHEMA_VERSION)
        .config(config)
        .layers(layers)
        .build()?;

    let mut manifest = fs::File::create(output.join("index.json"))?;
    image_manifest.to_writer(&mut manifest)?;

    Ok(())
}

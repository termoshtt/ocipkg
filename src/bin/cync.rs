use oci_spec::image::{
    Descriptor, DescriptorBuilder, ImageManifestBuilder, MediaType, SCHEMA_VERSION,
};
use sha2::{Digest, Sha256};
use std::{fs, io::Write, path::PathBuf};
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

fn calc_digest(buf: &[u8]) -> String {
    let hash = Sha256::digest(&buf);
    let hex_hash = base16ct::lower::encode_string(&hash);
    let digest = format!("sha256:{}", hex_hash);
    digest
}

fn main() -> anyhow::Result<()> {
    let Opt {
        input_directory,
        mut output,
    } = Opt::from_args();
    if !input_directory.is_dir() {
        panic!(
            "Input directory is not a directory: {}",
            input_directory
                .to_str()
                .expect("Non-UTF8 input is not supported")
        );
    }

    let config = DescriptorBuilder::default()
        .media_type(MediaType::ImageConfig)
        .size(7023)
        .digest("sha256:b5b2b2c507a0944348e0303114d8d93aaaa081732b86451d9bce1f432a537bc7")
        .build()?;

    let mut ar = tar::Builder::new(Vec::new());
    ar.append_dir_all("rootfs-c9d-v1", &input_directory)?;
    let buf: Vec<u8> = ar.into_inner()?;

    output.set_extension("tar");
    let mut out = fs::File::create(output)?;
    out.write_all(&buf)?;

    let layers: Vec<Descriptor> = vec![DescriptorBuilder::default()
        .media_type(MediaType::ImageLayer)
        .size(buf.len() as i64)
        .digest(calc_digest(&buf))
        .build()?];

    let image_manifest = ImageManifestBuilder::default()
        .schema_version(SCHEMA_VERSION)
        .config(config)
        .layers(layers)
        .build()?;

    image_manifest.to_writer_pretty(&mut std::io::stdout())?;

    Ok(())
}

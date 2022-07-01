use anyhow::Context;
use cargo_metadata::{Metadata, MetadataCommand};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "cargo-ocipkg")]
enum Opt {
    /// Build library or executable, and pack as a container
    Build {
        #[structopt(long)]
        release: bool,
    },

    /// Push container to OCI registry
    Publish {},
}

fn get_metadata() -> anyhow::Result<Metadata> {
    let mut args = std::env::args().skip_while(|val| !val.starts_with("--manifest-path"));
    let mut cmd = MetadataCommand::new();
    match args.next() {
        Some(ref p) if p == "--manifest-path" => {
            cmd.manifest_path(args.next().context("Manifest path not found")?);
        }
        Some(p) => {
            cmd.manifest_path(p.trim_start_matches("--manifest-path="));
        }
        None => {}
    };
    let metadata = cmd.exec()?;
    Ok(metadata)
}

fn main() -> anyhow::Result<()> {
    match Opt::from_args() {
        Opt::Build { release } => {
            dbg!(get_metadata()?, release);
            todo!("cargo-ocipkg build")
        }
        Opt::Publish {} => {
            dbg!(get_metadata()?);
            todo!("cargo-ocipkg publish")
        }
    }
}

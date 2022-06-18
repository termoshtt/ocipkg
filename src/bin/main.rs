use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "ocipkg", about = "OCI Registry for binary distribution")]
struct Opt {
    /// Input directory
    #[structopt(parse(from_os_str))]
    input_directory: PathBuf,

    /// Output oci archive
    #[structopt(parse(from_os_str))]
    output: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let Opt {
        input_directory,
        output,
    } = Opt::from_args();
    ocipkg::compose::compose(&input_directory, &output)?;
    Ok(())
}

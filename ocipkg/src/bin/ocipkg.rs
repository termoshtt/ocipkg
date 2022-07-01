use std::{fs, path::PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "ocipkg", about = "OCI Registry for binary distribution")]
enum Opt {
    /// Pack a directory into an oci-archive tar file
    Pack {
        /// Input directory
        #[structopt(parse(from_os_str))]
        input_directory: PathBuf,

        /// Output oci archive
        #[structopt(parse(from_os_str))]
        output: PathBuf,
    },

    /// Load and expand container local cache
    Load {
        /// Input oci-archive
        #[structopt(parse(from_os_str))]
        input: PathBuf,
    },

    /// Get and save in local storage
    Get { image_name: String },

    /// Get image directory to be used by ocipkg for given container name
    ImageDirectory { name: String },
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    match Opt::from_args() {
        Opt::Pack {
            input_directory,
            output,
        } => {
            let mut output = output;
            output.set_extension("tar");
            if output.exists() {
                anyhow::bail!("Output already exists");
            }
            let mut oci_archive = fs::File::create(output)?;
            ocipkg::image::pack_dir(&input_directory, &mut oci_archive)?;
        }

        Opt::Load { input } => {
            ocipkg::image::load(&input)?;
        }

        Opt::Get { image_name } => {
            let image_name = ocipkg::ImageName::parse(&image_name)?;
            ocipkg::distribution::get_image(&image_name).await?;
        }

        Opt::ImageDirectory { name } => {
            println!("{}", ocipkg::config::image_dir(&name)?.display());
        }
    }
    Ok(())
}

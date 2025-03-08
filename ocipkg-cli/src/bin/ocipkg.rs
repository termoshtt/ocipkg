use anyhow::{bail, Context, Result};
use clap::Parser;
use ocipkg::image::{Artifact, Image};
use std::path::*;

#[derive(Debug, Parser)]
#[command(version)]
enum Opt {
    /// Pack a directory into an oci-archive tar file
    Pack {
        /// Path of input directory to be packed
        input_directory: PathBuf,

        /// Path of output tar archive in oci-archive format
        output: PathBuf,

        /// Name of container, use UUID v4 hyphenated if not set.
        #[arg(short = 't', long = "tag")]
        tag: Option<String>,
    },

    /// Compose files into an oci-archive tar file
    Compose {
        /// Path of input file to be packed
        inputs: Vec<PathBuf>,

        /// Path of output tar archive in oci-archive format
        #[arg(short = 'o', long = "output")]
        output: PathBuf,

        /// Name of container, use UUID v4 hyphenated if not set.
        #[arg(short = 't', long = "tag")]
        tag: Option<String>,
    },

    /// Compose a static-linked executable file into an oci-archive tar file
    Runnable {
        /// Path of static-linked exetuable file
        input: PathBuf,

        /// Path of output tar archive in oci-archive format
        #[arg(short = 'o', long = "output")]
        output: PathBuf,

        /// Name of container, use UUID v4 hyphenated if not set.
        #[arg(short = 't', long = "tag")]
        tag: Option<String>,
    },

    /// Load and expand container local cache
    Load {
        /// Input oci-archive
        input: PathBuf,

        /// Overwrite existing local cache
        #[clap(short = 'f', long = "overwrite")]
        overwrite: bool,
    },

    /// Get and save in local storage
    Get {
        image_name: String,
        #[clap(short = 'f', long = "overwrite")]
        overwrite: bool,
    },

    /// Push oci-archive to registry
    Push {
        /// Input oci-archive
        input: PathBuf,
    },

    /// Get image directory to be used by ocipkg for given container name
    ImageDirectory {
        image_name: String,
    },

    List,

    /// Login to OCI registry
    Login {
        /// OCI registry to be login
        registry: String,
        #[clap(short = 'u', long = "username")]
        username: Option<String>,
        #[clap(short = 'p', long = "password")]
        password: Option<String>,
    },

    /// Inspect components in OCI archive
    Inspect {
        /// Input oci-archive
        input: PathBuf,
    },
}

fn main() -> Result<()> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    match Opt::parse() {
        Opt::Pack {
            input_directory,
            output,
            tag,
        } => {
            let mut output = output;
            output.set_extension("tar");
            let image_name = if let Some(name) = tag {
                ocipkg::ImageName::parse(&name)?
            } else {
                ocipkg::ImageName::default()
            };
            let mut b = ocipkg::image::Builder::new(output, image_name)?;
            b.append_dir_all(&input_directory)?;
            let _artifact = b.build()?;
        }

        Opt::Compose {
            inputs,
            output,
            tag,
        } => {
            let mut output = output;
            output.set_extension("tar");
            let image_name = if let Some(name) = tag {
                ocipkg::ImageName::parse(&name)?
            } else {
                ocipkg::ImageName::default()
            };
            let mut b = ocipkg::image::Builder::new(output, image_name)?;
            b.append_files(&inputs)?;
            let _artifact = b.build()?;
        }

        Opt::Runnable { input, output, tag } => {
            let mut output = output;
            output.set_extension("tar");
            let image_name = if let Some(name) = tag {
                ocipkg::ImageName::parse(&name)?
            } else {
                ocipkg::ImageName::default()
            };

            let _b = ocipkg::image::RunnableBuilder::new_archive(output, image_name)?;

            dbg!(input);

            todo!()
        }

        Opt::Load { input, overwrite } => {
            ocipkg::image::load(&input, overwrite)?;
        }

        Opt::Get {
            image_name,
            overwrite,
        } => {
            let image_name = ocipkg::ImageName::parse(&image_name)?;
            ocipkg::distribution::get_image(&image_name, overwrite)?;
        }

        Opt::Push { input } => {
            ocipkg::distribution::push_image(&input)?;
        }

        Opt::ImageDirectory { image_name } => {
            let image_name = ocipkg::ImageName::parse(&image_name)?;
            println!("{}", ocipkg::local::image_dir(&image_name)?.display());
        }

        Opt::List => {
            let images = ocipkg::local::get_image_list()?;
            for image in images {
                println!("{}", image);
            }
        }

        Opt::Login {
            registry,
            username,
            password,
        } => {
            let url = url::Url::parse(&registry)?;
            let mut auth = ocipkg::distribution::StoredAuth::load().unwrap_or_default();
            match (username, password) {
                (Some(username), Some(password)) => {
                    auth.add(
                        url.domain().context("URL does not contain domain name")?,
                        &username,
                        &password,
                    );
                }
                (None, None) => {}
                _ => {
                    bail!("Both username and password must be set");
                }
            }

            let _token = auth.get_token(&url)?;
            log::info!("Login succeed");
            auth.save()?;
        }

        Opt::Inspect { input } => {
            let mut ar = Artifact::from_oci_archive(&input)?;
            let image_name = ar.get_name()?;
            println!("[{image_name}]");
            let files = ar.files()?;
            for (i, path) in files.iter().enumerate() {
                if i < files.len() - 1 {
                    println!("  ├─ {}", path.display());
                } else {
                    println!("  └─ {}", path.display());
                }
            }
        }
    }
    Ok(())
}

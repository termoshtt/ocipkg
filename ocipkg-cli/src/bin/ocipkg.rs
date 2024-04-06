use base64::{engine::general_purpose::STANDARD, Engine};
use clap::Parser;
use flate2::read::GzDecoder;
use oci_spec::image::MediaType;
use ocipkg::error::*;
use std::{fs, path::*};

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

        /// Path to annotations file.
        #[arg(default_value = "ocipkg.toml")]
        annotations: PathBuf,
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

        /// Path to annotations file.
        #[arg(long = "annotations", default_value = "ocipkg.toml")]
        annotations: PathBuf,
    },

    /// Load and expand container local cache
    Load {
        /// Input oci-archive
        input: PathBuf,
    },

    /// Get and save in local storage
    Get {
        image_name: String,
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
        /// OCI registry to be logined
        registry: String,
        #[clap(short = 'u', long = "--username")]
        username: String,
        #[clap(short = 'p', long = "--password")]
        password: String,
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
            annotations,
        } => {
            let mut output = output;
            output.set_extension("tar");
            let f = fs::File::create(output)?;
            let mut b = ocipkg::image::Builder::new(f);
            if let Some(name) = tag {
                b.set_name(&ocipkg::ImageName::parse(&name)?);
            }
            if annotations.is_file() {
                let f = fs::read(annotations)?;
                let input = String::from_utf8(f).expect("Non-UTF8 string in TOML");
                b.set_annotations(
                    ocipkg::image::annotations::nested::Annotations::from_toml(&input)?.into(),
                )
            }
            b.append_dir_all(&input_directory)?;
            let _output = b.into_inner()?;
        }

        Opt::Compose {
            inputs,
            output,
            tag,
            annotations,
        } => {
            let mut output = output;
            output.set_extension("tar");
            let f = fs::File::create(output)?;
            let mut b = ocipkg::image::Builder::new(f);
            if let Some(name) = tag {
                b.set_name(&ocipkg::ImageName::parse(&name)?);
            }
            if annotations.is_file() {
                let f = fs::read(annotations)?;
                let input = String::from_utf8(f).expect("Non-UTF8 string in TOML");
                b.set_annotations(
                    ocipkg::image::annotations::nested::Annotations::from_toml(&input)?.into(),
                )
            }
            b.append_files(&inputs)?;
        }

        Opt::Load { input } => {
            ocipkg::image::load(&input)?;
        }

        Opt::Get { image_name } => {
            let image_name = ocipkg::ImageName::parse(&image_name)?;
            ocipkg::distribution::get_image(&image_name)?;
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
            let octet = STANDARD.encode(format!("{}:{}", username, password,));
            let mut new_auth = ocipkg::distribution::StoredAuth::default();
            new_auth.insert(url.domain().unwrap(), octet);
            let _token = new_auth.get_token(&url)?;
            println!("Login succeed");

            let mut auth = ocipkg::distribution::StoredAuth::load()?;
            auth.append(new_auth)?;
            auth.save()?;
        }

        Opt::Inspect { input } => {
            let mut f = fs::File::open(&input)?;
            let mut ar = ocipkg::image::Archive::new(&mut f);
            for (name, manifest) in ar.get_manifests()? {
                println!("[{}]", name);
                for layer in manifest.layers() {
                    let digest = ocipkg::Digest::new(layer.digest())?;
                    let entry = ar.get_blob(&digest)?;
                    match layer.media_type() {
                        MediaType::ImageLayerGzip => {
                            let buf = GzDecoder::new(entry);
                            let mut ar = tar::Archive::new(buf);
                            let paths: Vec<_> = ar
                                .entries()?
                                .filter_map(|entry| Some(entry.ok()?.path().ok()?.to_path_buf()))
                                .collect();
                            for (i, path) in paths.iter().enumerate() {
                                if i < paths.len() - 1 {
                                    println!("  ├─ {}", path.display());
                                } else {
                                    println!("  └─ {}", path.display());
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(())
}

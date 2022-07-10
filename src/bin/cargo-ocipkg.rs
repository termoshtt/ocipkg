use anyhow::{bail, Context};
use cargo_metadata::{Metadata, MetadataCommand, Package};
use clap::*;
use ocipkg::ImageName;
use std::{fs, path::PathBuf, process::Command};

#[derive(Parser, Debug)]
#[clap(version)]
enum Opt {
    #[clap(subcommand)]
    Ocipkg(Ocipkg),
}

#[derive(Subcommand, Debug)]
#[clap(version)]
enum Ocipkg {
    /// Build library or executable, and pack as a container
    Build {
        #[clap(long)]
        release: bool,
        #[clap(short = 'p', long = "package-name")]
        package_name: Option<String>,
        /// Name of container
        #[clap(short = 't', long = "tag")]
        tag: Option<String>,
    },
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

/// `-p`|`--package-name` option has higher priority than current directory
fn get_package(metadata: &Metadata, package_name: Option<String>) -> anyhow::Result<Package> {
    if let Some(name) = package_name {
        for pkg in metadata.workspace_packages() {
            if pkg.name == name {
                return Ok(pkg.clone());
            }
        }
    }
    if let Some(pkg) = metadata.root_package() {
        return Ok(pkg.clone());
    }
    bail!("Target package is not specified.")
}

fn get_build_dir(metadata: &Metadata, release: bool) -> PathBuf {
    let target_dir = metadata.target_directory.clone().into_std_path_buf();
    if release {
        target_dir.join("release")
    } else {
        target_dir.join("debug")
    }
}

fn generate_image_name(package: &Package) -> anyhow::Result<ImageName> {
    use serde_json::Value;
    match &package.metadata {
        Value::Object(obj) => {
            match obj
                .get("ocipkg")
                .context("`package.metadata.ocipkg` is missing")?
            {
                Value::Object(obj) => {
                    if let Value::String(ref image_name) = obj
                        .get("registry")
                        .context("`package.metadata.ocipkg` does not have `registry`")?
                    {
                        let image_name = ImageName::parse(image_name)?;
                        Ok(image_name)
                    } else {
                        bail!("`package.metadata.ocipkg.registry` must be a string")
                    }
                }
                _ => bail!("`package.metadata.ocipkg` must be a map"),
            }
        }
        _ => bail!("`package.metadata` must be object"),
    }
}

fn main() -> anyhow::Result<()> {
    match Opt::from_args() {
        Opt::Ocipkg(Ocipkg::Build {
            package_name,
            release,
            tag,
        }) => {
            let metadata = get_metadata()?;
            let package = get_package(&metadata, package_name)?;
            let build_dir = get_build_dir(&metadata, release);
            let image_name = if let Some(ref tag) = tag {
                ImageName::parse(&tag)?
            } else {
                generate_image_name(&package)?
            };

            let mut cmd = Command::new("cargo");
            cmd.arg("build");
            if release {
                cmd.arg("--release");
            }
            cmd.args(["--manifest-path", package.manifest_path.as_str()])
                .status()?;

            for target in package.targets {
                let mut targets = Vec::new();
                for ty in target.crate_types {
                    // FIXME support non-Linux OS
                    match ty.as_str() {
                        "staticlib" => {
                            targets.push(
                                build_dir.join(format!("lib{}.a", target.name.replace('-', "_"))),
                            );
                        }
                        "cdylib" => {
                            targets.push(
                                build_dir.join(format!("lib{}.so", target.name.replace('-', "_"))),
                            );
                        }
                        _ => {}
                    }
                }

                if targets.is_empty() {
                    bail!("No target exists for packing. Only staticlib or cdylib are suppoted.");
                }

                let dest = build_dir.join(format!("{}.tar", target.name));
                let f = fs::File::create(dest)?;
                let mut b = ocipkg::image::Builder::new(f);
                b.set_name(&image_name)?;
                let cfg = oci_spec::image::ImageConfigurationBuilder::default().build()?;
                b.append_config(cfg)?;
                b.append_files(&targets)?;
                let _output = b.into_inner()?;
            }
        }
    }
    Ok(())
}

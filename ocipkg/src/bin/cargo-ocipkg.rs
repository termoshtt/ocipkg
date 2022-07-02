use anyhow::{bail, Context};
use cargo_metadata::{Metadata, MetadataCommand, Package};
use std::{fs, path::PathBuf, process::Command};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "cargo-ocipkg")]
enum Opt {
    /// Build library or executable, and pack as a container
    Build {
        #[structopt(long)]
        release: bool,

        #[structopt(short = "p", long = "package-name")]
        package_name: Option<String>,

        /// Name of container, use UUID v4 hyphenated if not set.
        #[structopt(short = "t", long = "tag")]
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

fn main() -> anyhow::Result<()> {
    match Opt::from_args() {
        Opt::Build {
            package_name,
            release,
            tag,
        } => {
            let metadata = get_metadata()?;
            let package = get_package(&metadata, package_name)?;
            let build_dir = get_build_dir(&metadata, release);

            Command::new("cargo")
                .arg("build")
                .args(["--manifest-path", package.manifest_path.as_str()])
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
                if let Some(ref name) = tag {
                    b.set_name(name)?;
                }
                let cfg = oci_spec::image::ImageConfigurationBuilder::default().build()?;
                b.append_config(cfg)?;
                b.append_files(&targets)?;
                let _output = b.into_inner()?;
            }
        }
    }
    Ok(())
}

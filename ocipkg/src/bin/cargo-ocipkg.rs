use anyhow::{bail, Context};
use cargo_metadata::{Metadata, MetadataCommand, Package};
use std::{path::PathBuf, process::Command};
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
    },

    /// Push container to OCI registry
    Publish {
        #[structopt(long)]
        release: bool,

        #[structopt(short = "p", long = "package-name")]
        package_name: Option<String>,
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
        } => {
            let metadata = get_metadata()?;
            let package = get_package(&metadata, package_name)?;
            let build_dir = get_build_dir(&metadata, release);

            Command::new("cargo")
                .arg("build")
                .args(["--manifest-path", package.manifest_path.as_str()])
                .status()?;

            let mut targets = Vec::new();
            for target in package.targets {
                for ty in target.crate_types {
                    // FIXME support non-Linux OS
                    match ty.as_str() {
                        "staticlib" => {
                            targets.push(
                                build_dir.join(format!("lib{}.a", target.name.replace("-", "_"))),
                            );
                        }
                        "cdylib" => {
                            targets.push(
                                build_dir.join(format!("lib{}.so", target.name.replace("-", "_"))),
                            );
                        }
                        _ => {}
                    }
                }
            }
            dbg!(targets);
        }
        Opt::Publish {
            package_name,
            release,
        } => {
            let metadata = get_metadata()?;
            let package = get_package(&metadata, package_name)?;
            let build_dir = get_build_dir(&metadata, release);
            dbg!(package, build_dir);
            todo!("cargo-ocipkg publish");
        }
    }
    Ok(())
}

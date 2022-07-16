//! Executables and Rust crate for handling OCI images without container runtime.
//!
//! See [README.md at GitHub](https://github.com/termoshtt/ocipkg) for usage of the executables.
//! This reference describes the crate part.
//!

pub mod distribution;
pub mod error;
pub mod image;
pub mod local;

mod digest;
mod image_name;

pub use digest::Digest;
pub use image_name::ImageName;

use crate::error::*;
use std::fs;

/// Get and link package in `build.rs` with [cargo link instructions](https://doc.rust-lang.org/cargo/reference/build-scripts.html#outputs-of-the-build-script).
///
/// This is aimed to use in [build script](https://doc.rust-lang.org/cargo/reference/build-scripts.html) a.k.a. `build.rs`.
pub fn link_package(image_name: &str) -> Result<()> {
    let image_name = ImageName::parse(image_name)?;
    let dir = local::image_dir(&image_name)?;
    if !dir.exists() {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            distribution::get_image(&image_name)
                .await
                .expect("Failed to get image");
        });
    }
    println!("cargo:rustc-link-search={}", dir.display());
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let name = path
                .file_stem()
                .unwrap()
                .to_str()
                .expect("Non UTF-8 is not supported");
            let name = if let Some(name) = name.strip_prefix("lib") {
                name
            } else {
                continue;
            };
            if let Some(ext) = path.extension() {
                if ext == "a" {
                    println!("cargo:rustc-link-lib=static={}", name);
                }
                if ext == "so" {
                    println!("cargo:rustc-link-lib=dylib={}", name);
                }
            }
        }
    }
    println!("cargo:rerun-if-changed={}", dir.display());
    println!("cargo:rerun-if-env-changed=XDG_DATA_HOME");
    Ok(())
}

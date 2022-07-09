//! ocipkg
//! =======

pub mod config;
pub mod distribution;
pub mod image;

mod digest;
mod image_name;

pub use digest::Digest;
pub use image_name::ImageName;

use std::fs;

/// Get and link package to current crate using [cargo link instructions](https://doc.rust-lang.org/cargo/reference/build-scripts.html#outputs-of-the-build-script).
///
/// This is aimed to use in [build script](https://doc.rust-lang.org/cargo/reference/build-scripts.html) a.k.a. `build.rs`.
pub fn link_package(image_name: &str) -> anyhow::Result<()> {
    let image_name = ImageName::parse(image_name)?;
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        distribution::get_image(&image_name)
            .await
            .expect("Failed to get image");
    });
    let dir = config::image_dir(&image_name)?;
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
    Ok(())
}

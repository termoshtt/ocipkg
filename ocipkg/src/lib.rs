//! ocipkg
//! =======

pub mod config;
pub mod distribution;
pub mod image;

mod digest;
mod image_name;

pub use digest::Digest;
pub use image_name::ImageName;

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
    Ok(())
}

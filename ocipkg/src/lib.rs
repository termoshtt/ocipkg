//! ocipkg
//! =======

pub mod config;
pub mod distribution;
pub mod image;

mod digest;
mod image_name;

pub use digest::Digest;
pub use image_name::ImageName;

pub fn find_package(image_name: &str) -> anyhow::Result<()> {
    let image_name = ImageName::parse(image_name)?;
    dbg!(image_name);
    Ok(())
}

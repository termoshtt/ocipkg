//! Pull and Push images to OCI registry based on [OCI distribution specification](https://github.com/opencontainers/distribution-spec)

use crate::{
    image::{copy, Artifact, Image, OciArchive, RemoteBuilder},
    ImageName,
};
use anyhow::Result;
use std::path::Path;

mod auth;
mod client;

pub use auth::*;
pub use client::Client;
pub use oci_spec::image::MediaType;

/// Push image to registry
pub fn push_image(path: &Path) -> Result<()> {
    let mut oci_archive = OciArchive::new(path)?;
    let image_name = oci_archive.get_name()?;
    let remote = RemoteBuilder::new(image_name)?;
    copy(&mut oci_archive, remote)?;
    Ok(())
}

/// Get image from registry and save it into local storage
pub fn get_image(image_name: &ImageName, overwrite: bool) -> Result<()> {
    let mut artifact = Artifact::from_remote(image_name.clone())?;
    artifact.unpack(overwrite)?;
    Ok(())
}

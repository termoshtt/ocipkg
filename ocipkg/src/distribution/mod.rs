//! Pull and Push images to OCI registry based on [OCI distribution specification](https://github.com/opencontainers/distribution-spec)

mod auth;
mod client;
mod name;
mod reference;

pub use auth::*;
pub use client::Client;
pub use name::Name;
pub use oci_spec::image::MediaType;
pub use reference::Reference;

use crate::{
    image::{copy, Artifact, Image, OciArchive, RemoteBuilder},
    Digest, ImageName,
};
use anyhow::Result;
use std::{io::Read, path::Path};

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

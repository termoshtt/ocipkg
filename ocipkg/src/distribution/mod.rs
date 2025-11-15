//! Pull and Push images to OCI registry based on [OCI distribution specification](https://github.com/opencontainers/distribution-spec)

#[cfg(feature = "remote")]
mod auth;
#[cfg(feature = "remote")]
mod client;
mod name;
mod reference;

#[cfg(feature = "remote")]
pub use auth::*;
#[cfg(feature = "remote")]
pub use client::Client;
pub use name::Name;
pub use oci_spec::image::MediaType;
pub use reference::Reference;

#[cfg(feature = "remote")]
use crate::{
    image::{copy, Artifact, Image, OciArchive, RemoteBuilder},
    ImageName,
};
#[cfg(feature = "remote")]
use anyhow::Result;
#[cfg(feature = "remote")]
use oci_spec::image::Digest;
#[cfg(feature = "remote")]
use std::{io::Read, path::Path};

/// Push image to registry
#[cfg(feature = "remote")]
pub fn push_image(path: &Path) -> Result<()> {
    let mut oci_archive = OciArchive::new(path)?;
    let image_name = oci_archive.get_name()?;
    let remote = RemoteBuilder::new(image_name)?;
    copy(&mut oci_archive, remote)?;
    Ok(())
}

/// Get image from registry and save it into local storage
#[cfg(feature = "remote")]
pub fn get_image(image_name: &ImageName, overwrite: bool) -> Result<()> {
    let mut artifact = Artifact::from_remote(image_name.clone())?;
    artifact.unpack(overwrite)?;
    Ok(())
}

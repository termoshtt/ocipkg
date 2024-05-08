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
    image::{Artifact, ImageLayout, ImageLayoutBuilder, OciDirBuilder},
    media_types, Digest, ImageName,
};
use anyhow::{bail, Result};
use std::{fs, io::Read, path::Path};

/// Push image to registry
pub fn push_image(path: &Path) -> Result<()> {
    if !path.is_file() {
        bail!("{} is not a file", path.display());
    }
    let mut ar = Artifact::from_oci_archive(path)?;
    let (image_name, manifest) = ar.get_manifest()?;
    let Some(image_name) = image_name else {
        bail!("Missing image name in manifest");
    };
    log::info!("Push image: {}", image_name);

    let mut client = Client::new(image_name.registry_url()?, image_name.name)?;
    for (_, blob) in ar.get_layers()? {
        client.push_blob(&blob)?;
    }
    let (_, blob) = ar.get_config()?;
    client.push_blob(&blob)?;
    client.push_manifest(&image_name.reference, &manifest)?;

    Ok(())
}

/// Get image from registry and save it into local storage
pub fn get_image(image_name: &ImageName, overwrite: bool) -> Result<()> {
    let dest = crate::local::image_dir(image_name)?;
    if dest.exists() {
        if overwrite {
            log::info!("Remove existing image: {}", dest.display());
            fs::remove_dir_all(&dest)?;
        } else {
            bail!("Image already exists: {}", image_name);
        }
    }
    let mut oci_dir = OciDirBuilder::new(dest.join(".oci-dir"))?;

    let mut client = Client::from_image_name(image_name)?;

    let manifest = client.get_manifest(&image_name.reference)?;

    if *manifest.config().media_type() != MediaType::EmptyJSON {
        let digest = Digest::new(manifest.config().digest())?;
        let blob = client.get_blob(&digest)?;
        oci_dir.add_blob(&blob)?;
    }

    for desc in manifest.layers() {
        let digest = Digest::new(desc.digest())?;
        let blob = client.get_blob(&digest)?;
        oci_dir.add_blob(&blob)?;

        match desc.media_type() {
            // For compatibility to 0.2.x
            MediaType::ImageLayerGzip => {
                log::warn!(
                    "{} is deprecated. Use OCI Artifact based container.",
                    desc.media_type()
                );
                let buf = flate2::read::GzDecoder::new(blob.as_slice());
                tar::Archive::new(buf).unpack(&dest)?;
            }
            MediaType::ImageLayer => {
                log::warn!(
                    "{} is deprecated. Use OCI Artifact based container.",
                    desc.media_type()
                );
                let buf = blob.as_slice();
                tar::Archive::new(buf).unpack(&dest)?;
            }

            // OCI Artifact based (0.3.0+)
            media_type @ MediaType::Other(_) if media_type == &media_types::layer_tar_gzip() => {
                todo!()
            }
            _ => {}
        }
    }
    oci_dir.build(manifest, image_name.clone())?;

    Ok(())
}

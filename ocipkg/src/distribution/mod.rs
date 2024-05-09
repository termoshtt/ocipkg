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
    image::{copy, Image, OciArchive, RemoteBuilder},
    Digest, ImageName,
};
use anyhow::{bail, Result};
use std::{fs, io::Read, path::Path};

/// Push image to registry
pub fn push_image(path: &Path) -> Result<()> {
    let mut oci_archive = OciArchive::new(path)?;
    let image_name = oci_archive.get_name()?;
    let remote = RemoteBuilder::new(image_name)?;
    copy(oci_archive, remote)?;
    Ok(())
}

/// Get image from registry and save it into local storage
pub fn get_image(image_name: &ImageName, overwrite: bool) -> Result<()> {
    let dest = crate::local::image_dir(image_name)?;
    if dest.exists() {
        if overwrite {
            fs::remove_dir_all(&dest)?;
        } else {
            bail!("Image already exists: {}", image_name);
        }
    }
    let blob_root = dest.join(".blob");
    fs::create_dir_all(&blob_root)?;

    let mut client = Client::from_image_name(image_name)?;

    log::info!("Get manifest: {}", image_name);
    let manifest = client.get_manifest(&image_name.reference)?;
    fs::write(
        dest.join(".manifest.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;

    for desc in manifest.layers() {
        let digest = Digest::new(desc.digest())?;
        let dest_algorithm = blob_root.join(&digest.algorithm);
        fs::create_dir_all(&dest_algorithm)?;
        let blob_path = dest_algorithm.join(&digest.encoded);
        log::info!("Get blob: {}", digest);
        let blob = client.get_blob(&digest)?;
        fs::write(&blob_path, &blob)?;

        match desc.media_type() {
            MediaType::ImageLayerGzip => {
                let buf = flate2::read::GzDecoder::new(blob.as_slice());
                tar::Archive::new(buf).unpack(&dest)?;
            }
            MediaType::ImageLayer => {
                let buf = blob.as_slice();
                tar::Archive::new(buf).unpack(&dest)?;
            }
            _ => {}
        }
    }

    Ok(())
}

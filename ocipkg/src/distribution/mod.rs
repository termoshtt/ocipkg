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

use crate::{error::*, Digest, ImageName};
use std::{fs, io::Read, path::Path};

/// Push image to registry
pub fn push_image(path: &Path) -> Result<()> {
    if !path.is_file() {
        return Err(Error::NotAFile(path.to_owned()));
    }
    let mut f = fs::File::open(path)?;
    let mut ar = crate::image::Archive::new(&mut f);
    for (image_name, manifest) in ar.get_manifests()? {
        log::info!("Push image: {}", image_name);
        let mut client = Client::new(image_name.registry_url()?, image_name.name)?;
        for layer in manifest.layers() {
            let digest = Digest::new(layer.digest())?;
            let mut entry = ar.get_blob(&digest)?;
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf)?;
            client.push_blob(&buf)?;
        }
        let digest = Digest::new(manifest.config().digest())?;
        let mut entry = ar.get_blob(&digest)?;
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf)?;
        client.push_blob(&buf)?;
        client.push_manifest(&image_name.reference, &manifest)?;
    }
    Ok(())
}

/// Get image from registry and save it into local storage
pub fn get_image(image_name: &ImageName, overwrite: bool) -> Result<()> {
    let dest = crate::local::image_dir(image_name)?;
    if dest.exists() {
        if overwrite {
            fs::remove_dir_all(&dest)?;
        } else {
            return Err(Error::ImageAlreadyExists(dest));
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

    if *manifest.config().media_type() != MediaType::EmptyJSON {
        let digest = Digest::new(manifest.config().digest())?;
        log::info!("Get config: {}", digest);
        let blob = client.get_blob(&digest)?;
        fs::write(blob_root.join("config"), &blob)?;
    }

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

/// Get the data blob of a specific image layer, filtering by media_type.
pub fn get_layer_bytes(image_name: &ImageName, f: impl Fn(&MediaType) -> bool) -> Result<Vec<u8>> {
    let registry_url = image_name.registry_url()?;
    let mut client = Client::new(registry_url, image_name.name.clone())?;
    let manifest = client.get_manifest(&image_name.reference)?;
    dbg!(&manifest);
    let layer = manifest
        .layers()
        .iter()
        .find(|&d| f(d.media_type()))
        .ok_or(Error::MissingLayer)?;
    let digest = Digest::new(layer.digest())?;

    client.get_blob(&digest)
}

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

use crate::{error::*, oci_dir::OciDirBuilder, Digest, ImageName};
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
            log::info!("Remove existing image: {}", dest.display());
            fs::remove_dir_all(&dest)?;
        } else {
            return Err(Error::ImageAlreadyExists(dest));
        }
    }
    let oci_dir = OciDirBuilder::new(dest.join(".oci-dir"))?;

    let mut client = Client::from_image_name(image_name)?;

    let manifest = client.get_manifest(&image_name.reference)?;

    if *manifest.config().media_type() != MediaType::EmptyJSON {
        let digest = Digest::new(manifest.config().digest())?;
        let blob = client.get_blob(&digest)?;
        oci_dir.save_blob(&blob)?;
    }

    for desc in manifest.layers() {
        let digest = Digest::new(desc.digest())?;
        let blob = client.get_blob(&digest)?;
        oci_dir.save_blob(&blob)?;

        match desc.media_type() {
            // For compatiblity to 0.2.x
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
            MediaType::Other(t) if t == "application/vnd.ocipkg.file+gzip" => {
                let name = desc
                    .annotations()
                    .as_ref()
                    .and_then(|annotations| annotations.get("vnd.ocipkg.file.name"))
                    .ok_or(Error::MissingAnnotation)?;
                let mut decoder = flate2::read::GzDecoder::new(blob.as_slice());
                let mut buf = Vec::new();
                decoder.read_to_end(&mut buf)?;
                fs::write(dest.join(name), buf)?;
            }
            _ => {}
        }
    }
    oci_dir.finish(manifest)?;

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

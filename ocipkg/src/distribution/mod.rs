//! Rust binding to [OCI distribution spec](https://github.com/opencontainers/distribution-spec)

mod client;
mod name;
mod reference;

pub use client::Client;
pub use name::Name;
pub use reference::Reference;

use crate::ImageName;
use oci_spec::image::*;

/// Get image from registry and save it into local storage
pub async fn get_image(image_name: &ImageName) -> anyhow::Result<()> {
    let ImageName {
        name,
        domain,
        reference,
        ..
    } = image_name;
    let client = Client::new(&image_name.url(), name)?;
    let manifest = client.get_manifest(reference).await?;
    let dest = crate::config::image_dir(&format!(
        "{}/{}/__{}",
        domain,
        name.as_str(),
        reference.as_str()
    ))?;
    for layer in manifest.layers() {
        let blob = client.get_blob(layer.digest()).await?;
        match layer.media_type() {
            MediaType::ImageLayerGzip => {}
            MediaType::Other(ty) => {
                // application/vnd.docker.image.rootfs.diff.tar.gzip case
                if !ty.ends_with("tar.gzip") {
                    continue;
                }
            }
            _ => continue,
        }
        let buf = flate2::read::GzDecoder::new(blob.as_ref());
        tar::Archive::new(buf).unpack(dest)?;
        return Ok(());
    }
    anyhow::bail!("Layer not found")
}

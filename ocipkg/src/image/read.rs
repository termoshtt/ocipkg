use anyhow::Context;
use oci_spec::image::*;
use std::{
    fs,
    io::{Read, Seek},
    path::*,
};

use crate::digest::Digest;

/// Get index.json from oci-archive
fn get_index(input: &mut fs::File) -> anyhow::Result<ImageIndex> {
    input.rewind()?;
    let mut ar = tar::Archive::new(input);
    // Searched linearly since tar archive does not have offset table.
    for entry in ar.entries_with_seek()? {
        let mut entry = entry?;
        if entry.path()?.as_os_str() == "index.json" {
            let mut out = Vec::new();
            entry.read_to_end(&mut out)?;
            return Ok(ImageIndex::from_reader(&*out)?);
        }
    }
    anyhow::bail!("index.json not found in oci-archive")
}

/// Get manifests listed in index.json
fn get_manifests(input: &mut fs::File, descs: &[Descriptor]) -> anyhow::Result<Vec<ImageManifest>> {
    let mut manifests = Vec::new();
    input.rewind()?;
    let mut ar = tar::Archive::new(input);
    for entry in ar.entries_with_seek()? {
        let entry = entry?;
        for d in descs {
            let digest = Digest::new(d.digest())?;
            if entry.path()? == digest.as_path() {
                manifests.push(ImageManifest::from_reader(entry)?);
                break;
            }
        }
    }
    anyhow::ensure!(
        descs.len() == manifests.len(),
        "Some manifest not found in container"
    );
    Ok(manifests)
}

/// Get configuration specified in manifest
fn get_config(input: &mut fs::File, digest: &str) -> anyhow::Result<ImageConfiguration> {
    input.rewind()?;
    let mut ar = tar::Archive::new(input);
    for entry in ar.entries_with_seek()? {
        let entry = entry?;
        let digest = Digest::new(digest)?;
        if entry.path()? == digest.as_path() {
            return Ok(ImageConfiguration::from_reader(entry)?);
        }
    }
    anyhow::bail!("index.json not found in oci-archive")
}

fn expand_layer_at(input: &mut fs::File, layer: &Descriptor, dest: &Path) -> anyhow::Result<()> {
    input.rewind()?;
    let mut ar = tar::Archive::new(input);
    for entry in ar.entries_with_seek()? {
        let entry = entry?;
        let digest = Digest::new(layer.digest())?;
        if entry.path()? == digest.as_path() {
            match layer.media_type() {
                MediaType::ImageLayerGzip => {
                    let buf = flate2::read::GzDecoder::new(entry);
                    tar::Archive::new(buf).unpack(dest)?;
                    return Ok(());
                }
                _ => anyhow::bail!("Unsupported layer type"),
            }
        }
    }
    anyhow::bail!("Given digest not found in archive");
}

/// Load oci-archive into local storage
pub fn load(input: &Path) -> anyhow::Result<()> {
    if !input.exists() {
        anyhow::bail!("Input file does not exist");
    }
    let mut f = fs::File::open(input)?;
    let index = get_index(&mut f)?;

    let image_names = index
        .manifests()
        .iter()
        .map(|manifest| {
            let image_name = manifest
                .annotations()
                .as_ref()
                .context("annotations of manifest must exist")?
                .get("org.opencontainers.image.ref.name")
                .context("index.json does not has image name for some manifest")?;
            Ok(image_name.as_str())
        })
        .collect::<anyhow::Result<Vec<&str>>>()?;

    let manifests = get_manifests(&mut f, index.manifests())?;

    for (image_name, manifest) in image_names.iter().zip(&manifests) {
        let _cfg = get_config(&mut f, manifest.config().digest())?;
        let dest = crate::config::image_dir(image_name)?;
        if dest.exists() {
            continue;
        }
        fs::create_dir_all(&dest)?;
        for layer in manifest.layers() {
            expand_layer_at(&mut f, layer, &dest)?;
        }
    }

    Ok(())
}

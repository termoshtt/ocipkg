use anyhow::Context;
use oci_spec::image::*;
use std::{
    fs,
    io::{Read, Seek},
    path::*,
};

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

fn split_digest(digest: &str) -> anyhow::Result<(&str, &str)> {
    let mut iter = digest.split(":");
    match (iter.next(), iter.next()) {
        (Some(algorithm), Some(hash)) => Ok((algorithm, hash)),
        _ => anyhow::bail!("Invalid digest in index.json"),
    }
}

/// Check path of tar entry is in `blobs/{algorithm}/{hash}` form
fn match_digest(entry: &tar::Entry<&mut fs::File>, digest: &str) -> anyhow::Result<bool> {
    let path = entry.path()?;
    let mut iter = path.components();
    match (iter.next(), iter.next(), iter.next()) {
        (
            Some(Component::Normal(top)),
            Some(Component::Normal(algorithm)),
            Some(Component::Normal(hash)),
        ) => {
            let (a, h) = split_digest(digest)?;
            Ok(top == "blobs" && algorithm == a && hash == h)
        }
        _ => Ok(false),
    }
}

/// Get manifests listed in index.json
fn get_manifests(input: &mut fs::File, descs: &[Descriptor]) -> anyhow::Result<Vec<ImageManifest>> {
    let mut manifests = Vec::new();
    input.rewind()?;
    let mut ar = tar::Archive::new(input);
    for entry in ar.entries_with_seek()? {
        let entry = entry?;
        for d in descs {
            if match_digest(&entry, d.digest())? {
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
        if match_digest(&entry, digest)? {
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
        if match_digest(&entry, layer.digest())? {
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

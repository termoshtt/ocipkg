use anyhow::Context;
use oci_spec::image::*;
use std::{
    fs,
    io::{Read, Seek},
    path::*,
};

use crate::{digest::Digest, image::*, ImageName};

/// Handler for oci-archive tar file.
///
/// oci-archive consists of several manifests i.e. several containers.
pub struct Archive<'buf, W: Read + Seek> {
    archive: Option<tar::Archive<&'buf mut W>>,
}

impl<'buf, W: Read + Seek> Archive<'buf, W> {
    pub fn new(buf: &'buf mut W) -> Self {
        Self {
            archive: Some(tar::Archive::new(buf)),
        }
    }

    pub fn entries(&mut self) -> anyhow::Result<tar::Entries<&'buf mut W>> {
        let raw = self
            .archive
            .take()
            .expect("This never becomes None except in this function");
        let inner = raw.into_inner();
        inner.rewind()?;
        self.archive = Some(tar::Archive::new(inner));
        Ok(self
            .archive
            .as_mut()
            .expect("This never becomes None except in this function")
            .entries_with_seek()?)
    }

    pub fn get_manifests(&mut self) -> anyhow::Result<Vec<(ImageName, ImageManifest)>> {
        let index = self.get_index()?;
        index
            .manifests()
            .iter()
            .map(|manifest| {
                let annotations = Annotations::from_map(
                    manifest
                        .annotations()
                        .as_ref()
                        .context("annotations of manifest must exist")?,
                );
                let image_name = annotations
                    .ref_name
                    .context("index.json does not has image ref name")?;
                let image_name = ImageName::parse(&image_name)?;
                let digest = Digest::new(manifest.digest())?;
                let manifest = self.get_manifest(&digest)?;
                Ok((image_name, manifest))
            })
            .collect()
    }

    pub fn get_index(&mut self) -> anyhow::Result<ImageIndex> {
        for entry in self.entries()? {
            let mut entry = entry?;
            if entry.path()?.as_os_str() == "index.json" {
                let mut out = Vec::new();
                entry.read_to_end(&mut out)?;
                return Ok(ImageIndex::from_reader(&*out)?);
            }
        }
        anyhow::bail!("index.json not found")
    }

    pub fn get_blob(&mut self, digest: &Digest) -> anyhow::Result<tar::Entry<&'buf mut W>> {
        for entry in self.entries()? {
            let entry = entry?;
            if entry.path()? == digest.as_path() {
                return Ok(entry);
            }
        }
        anyhow::bail!("No blob found with digest: {}", digest)
    }

    pub fn get_manifest(&mut self, digest: &Digest) -> anyhow::Result<ImageManifest> {
        let entry = self.get_blob(digest)?;
        Ok(ImageManifest::from_reader(entry)?)
    }

    pub fn get_config(&mut self, digest: &Digest) -> anyhow::Result<ImageConfiguration> {
        let entry = self.get_blob(digest)?;
        Ok(ImageConfiguration::from_reader(entry)?)
    }

    pub fn unpack_layer(&mut self, layer: &Descriptor, dest_root: &Path) -> anyhow::Result<()> {
        let digest = Digest::new(layer.digest())?;
        let blob = self.get_blob(&digest)?;
        match layer.media_type() {
            MediaType::ImageLayerGzip => {
                let buf = flate2::read::GzDecoder::new(blob);
                let mut ar = tar::Archive::new(buf);
                for entry in ar.entries()? {
                    let mut entry = entry?;
                    let path = entry.path()?;
                    // Remove rootfs
                    let mut dest = dest_root.to_owned();
                    for c in path.components().skip(1) {
                        dest = dest.join(c);
                    }
                    entry.unpack(dest)?;
                }
                Ok(())
            }
            _ => anyhow::bail!("Unsupported layer type"),
        }
    }
}

/// Load oci-archive into local storage
pub fn load(input: &Path) -> anyhow::Result<()> {
    if !input.exists() {
        anyhow::bail!("Input file does not exist");
    }
    let mut f = fs::File::open(input)?;
    let mut ar = Archive::new(&mut f);
    for (image_name, manifest) in ar.get_manifests()? {
        let dest = crate::config::image_dir(&image_name)?;
        if dest.exists() {
            continue;
        }
        fs::create_dir_all(&dest)?;
        for layer in manifest.layers() {
            ar.unpack_layer(layer, &dest)?;
        }
    }
    Ok(())
}

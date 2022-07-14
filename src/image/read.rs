use crate::error::*;
use oci_spec::image::*;
use std::{
    fs,
    io::{Read, Seek},
    path::*,
};

use crate::{digest::Digest, image::*, ImageName};

/// Handler for oci-archive format
///
/// oci-archive consists of several manifests i.e. several container.
pub struct Archive<'buf, W: Read + Seek> {
    archive: Option<tar::Archive<&'buf mut W>>,
}

impl<'buf, W: Read + Seek> Archive<'buf, W> {
    pub fn new(buf: &'buf mut W) -> Self {
        Self {
            archive: Some(tar::Archive::new(buf)),
        }
    }

    pub fn entries(&mut self) -> Result<tar::Entries<&'buf mut W>> {
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

    pub fn get_manifests(&mut self) -> Result<Vec<(ImageName, ImageManifest)>> {
        let index = self.get_index()?;
        index
            .manifests()
            .iter()
            .map(|manifest| {
                let annotations =
                    Annotations::from_map(&manifest.annotations().clone().unwrap_or_default());
                let image_name = annotations.ref_name.ok_or(Error::MissingManifestName)?;
                let image_name = ImageName::parse(&image_name)?;
                let digest = Digest::new(manifest.digest())?;
                let manifest = self.get_manifest(&digest)?;
                Ok((image_name, manifest))
            })
            .collect()
    }

    pub fn get_index(&mut self) -> Result<ImageIndex> {
        for entry in self.entries()? {
            let mut entry = entry?;
            if entry.path()?.as_os_str() == "index.json" {
                let mut out = Vec::new();
                entry.read_to_end(&mut out)?;
                return Ok(ImageIndex::from_reader(&*out)?);
            }
        }
        Err(Error::MissingIndex)
    }

    pub fn get_blob(&mut self, digest: &Digest) -> Result<tar::Entry<&'buf mut W>> {
        for entry in self.entries()? {
            let entry = entry?;
            if entry.path()? == digest.as_path() {
                return Ok(entry);
            }
        }
        Err(Error::UnknownDigest(digest.clone()))
    }

    pub fn get_manifest(&mut self, digest: &Digest) -> Result<ImageManifest> {
        let entry = self.get_blob(digest)?;
        Ok(ImageManifest::from_reader(entry)?)
    }

    pub fn get_config(&mut self, digest: &Digest) -> Result<ImageConfiguration> {
        let entry = self.get_blob(digest)?;
        Ok(ImageConfiguration::from_reader(entry)?)
    }

    pub fn unpack_layer(&mut self, layer: &Descriptor, dest: &Path) -> Result<()> {
        let digest = Digest::new(layer.digest())?;
        let blob = self.get_blob(&digest)?;
        match layer.media_type() {
            MediaType::ImageLayerGzip => {
                let buf = flate2::read::GzDecoder::new(blob);
                tar::Archive::new(buf).unpack(dest)?;
                Ok(())
            }
            _ => unimplemented!("Unsupported layer type"),
        }
    }
}

/// Load oci-archive into local storage
pub fn load(input: &Path) -> Result<()> {
    let mut f = fs::File::open(input)?;
    let mut ar = Archive::new(&mut f);
    for (image_name, manifest) in ar.get_manifests()? {
        let dest = crate::config::image_dir(&image_name)?;
        if dest.exists() {
            log::warn!(
                "Local image aleady exists, skip loading: {}",
                dest.display()
            );
            continue;
        }
        log::info!("Create local image: {}", dest.display());
        fs::create_dir_all(&dest)?;
        for layer in manifest.layers() {
            ar.unpack_layer(layer, &dest)?;
        }
    }
    Ok(())
}

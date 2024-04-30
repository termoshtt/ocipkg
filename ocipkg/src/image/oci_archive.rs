use crate::{
    image::{ImageLayout, ImageLayoutBuilder},
    Digest,
};
use anyhow::{bail, Result};
use chrono::Utc;
use oci_spec::image::{DescriptorBuilder, ImageIndex, ImageIndexBuilder, ImageManifest, MediaType};
use std::{
    fs,
    io::{Read, Seek},
    path::{Path, PathBuf},
};

/// oci-archive, i.e. a tarball of a directory in the form of [OCI Image Layout specification](https://github.com/opencontainers/image-spec/blob/v1.1.0/image-layout.md)
pub struct OciArchiveBuilder {
    path: PathBuf,
    ar: tar::Builder<fs::File>,
}

impl OciArchiveBuilder {
    pub fn new(path: PathBuf) -> Result<Self> {
        if path.exists() {
            bail!("File already exists: {}", path.display());
        }
        let f = fs::File::create(&path)?;
        let ar = tar::Builder::new(f);
        Ok(Self { ar, path })
    }
}

impl ImageLayoutBuilder for OciArchiveBuilder {
    type ImageLayout = OciArchive;

    fn add_blob(&mut self, blob: &[u8]) -> Result<Digest> {
        let digest = Digest::from_buf_sha256(blob);
        self.ar
            .append_data(&mut create_file_header(blob.len()), digest.as_path(), blob)?;
        Ok(digest)
    }

    fn finish(mut self, manifest: ImageManifest) -> Result<Self::ImageLayout> {
        let manifest_json = serde_json::to_string(&manifest)?;
        let digest = self.add_blob(manifest_json.as_bytes())?;
        let descriptor = DescriptorBuilder::default()
            .media_type(MediaType::ImageManifest)
            .size(manifest_json.len() as i64)
            .digest(digest.to_string())
            .build()?;
        let index = ImageIndexBuilder::default()
            .schema_version(2_u32)
            .manifests(vec![descriptor])
            .build()?;
        let index_json = serde_json::to_string(&index)?;
        let buf = index_json.as_bytes();
        self.ar
            .append_data(&mut create_file_header(buf.len()), "index.json", buf)?;

        self.ar.finish()?;
        Ok(OciArchive::new(&self.path)?)
    }
}

fn create_file_header(size: usize) -> tar::Header {
    let mut header = tar::Header::new_gnu();
    header.set_size(size as u64);
    header.set_cksum();
    header.set_mode(0b110100100); // rw-r--r--
    header.set_mtime(Utc::now().timestamp() as u64);
    header
}

pub struct OciArchive {
    // Since `tar::Archive` does not have API to get mutable reference of inner part, we need to take it out and put it back.
    ar: Option<tar::Archive<fs::File>>,
}

impl OciArchive {
    pub fn new(path: &Path) -> Result<Self> {
        if !path.is_file() {
            bail!("Not a file: {}", path.display());
        }
        let f = fs::File::open(path)?;
        let ar = tar::Archive::new(f);
        Ok(Self { ar: Some(ar) })
    }

    fn rewind(&mut self) -> Result<()> {
        let ar = self.ar.take().unwrap();
        let mut f = ar.into_inner();
        f.rewind()?;
        self.ar = Some(tar::Archive::new(f));
        Ok(())
    }

    fn get_entries(&mut self) -> Result<impl Iterator<Item = tar::Entry<fs::File>>> {
        self.rewind()?;
        Ok(self
            .ar
            .as_mut()
            .unwrap()
            .entries_with_seek()?
            .filter_map(|e| e.ok()))
    }
}

impl ImageLayout for OciArchive {
    fn get_index(&mut self) -> Result<ImageIndex> {
        for entry in self.get_entries()? {
            let path = entry.path()?;
            if path == Path::new("index.json") {
                return Ok(ImageIndex::from_reader(entry)?);
            }
        }
        bail!("Missing index.json")
    }

    fn get_blob(&mut self, digest: &Digest) -> Result<Vec<u8>> {
        for mut entry in self.get_entries()? {
            let path = entry.path()?;
            if path == digest.as_path() {
                let mut buf = Vec::new();
                entry.read_to_end(&mut buf)?;
                return Ok(buf);
            }
        }
        bail!("Missing blob: {}", digest)
    }
}

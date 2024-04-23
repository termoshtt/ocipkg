use crate::{error::*, Digest};
use chrono::Utc;
use oci_spec::image::{DescriptorBuilder, ImageIndexBuilder, ImageManifest, MediaType};
use std::{fs, path::PathBuf};

/// oci-archive, i.e. a tarball of a directory in the form of [OCI Image Layout specification](https://github.com/opencontainers/image-spec/blob/v1.1.0/image-layout.md)
pub struct OciArchiveBuilder {
    ar: tar::Builder<fs::File>,
}

impl OciArchiveBuilder {
    pub fn new(out: PathBuf) -> Result<Self> {
        if out.exists() {
            return Err(Error::FileAlreadyExists(out));
        }
        let f = fs::File::create(&out)?;
        let ar = tar::Builder::new(f);
        Ok(Self { ar })
    }

    pub fn save_blob(&mut self, blob: &[u8]) -> Result<Digest> {
        let digest = Digest::from_buf_sha256(blob);
        self.ar
            .append_data(&mut create_file_header(blob.len()), digest.as_path(), blob)?;
        Ok(digest)
    }

    pub fn finish(mut self, manifest: ImageManifest) -> Result<()> {
        let manifest_json = serde_json::to_string(&manifest)?;
        let digest = self.save_blob(manifest_json.as_bytes())?;
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
        Ok(())
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

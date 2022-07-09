//! Compose directory as a container tar

use anyhow::{bail, Context};
use flate2::{write::GzEncoder, Compression};
use oci_spec::image::*;
use std::{collections::HashMap, convert::TryFrom, fs, io, path::Path, time::SystemTime};

use crate::Digest;

/// Build a container in oci-archive format based
/// on the [OCI image spec](https://github.com/opencontainers/image-spec)
pub struct Builder<W: io::Write> {
    /// Include a flag to check if finished
    builder: Option<tar::Builder<W>>,
    name: Option<String>,
    config: Option<Descriptor>,
    layers: Vec<Descriptor>,
}

impl<W: io::Write> Builder<W> {
    pub fn new(writer: W) -> Self {
        Builder {
            builder: Some(tar::Builder::new(writer)),
            name: None,
            config: None,
            layers: Vec::new(),
        }
    }

    /// Set name of container, used in `org.opencontainers.image.ref.name` tag.
    ///
    /// If not set, a random name using UUID v4 hyphenated is set.
    pub fn set_name(&mut self, name: &str) -> anyhow::Result<()> {
        if self.name.replace(name.to_string()).is_some() {
            bail!("Name is set twice.");
        }
        Ok(())
    }

    fn get_name(&self) -> String {
        self.name
            .clone()
            .unwrap_or(format!("{}", uuid::Uuid::new_v4().as_hyphenated()))
    }

    pub fn append_config(&mut self, cfg: ImageConfiguration) -> anyhow::Result<()> {
        let mut buf = Vec::new();
        cfg.to_writer(&mut buf)?;
        let config_desc = self.save_blob(MediaType::ImageConfig, &buf)?;
        if self.config.replace(config_desc).is_some() {
            bail!("ImageConfiguration is set twice.")
        } else {
            Ok(())
        }
    }

    /// Append a files as a layer
    pub fn append_files(&mut self, ps: &[impl AsRef<Path>]) -> anyhow::Result<()> {
        let encoder = GzEncoder::new(Vec::new(), Compression::default());
        let mut ar = tar::Builder::new(encoder);
        for path in ps {
            let path = path.as_ref();
            if !path.is_file() {
                bail!("Not a file, or not exist: {}", path.display());
            }
            let name = path
                .file_name()
                .expect("This never fails since checked above")
                .to_str()
                .expect("Non-UTF8 file name");
            let mut f = fs::File::open(path)?;
            ar.append_file(name, &mut f)
                .context("Error while reading input directory")?;
        }
        let buf = ar
            .into_inner()
            .expect("This never fails since tar arhive is creating on memory")
            .finish()
            .expect("This never fails since zip is creating on memory");
        let layer_desc = self.save_blob(MediaType::ImageLayerGzip, &buf)?;
        self.layers.push(layer_desc);
        Ok(())
    }

    /// Append directory as a layer
    pub fn append_dir_all(&mut self, path: &Path) -> anyhow::Result<()> {
        if !path.is_dir() {
            bail!("Not a directory, or not exist: {}", path.display());
        }
        let encoder = GzEncoder::new(Vec::new(), Compression::default());
        let mut ar = tar::Builder::new(encoder);
        ar.append_dir_all("", path)
            .context("Error while reading input directory")?;
        let buf = ar
            .into_inner()
            .expect("This never fails since tar arhive is creating on memory")
            .finish()
            .expect("This never fails since zip is creating on memory");
        let layer_desc = self.save_blob(MediaType::ImageLayerGzip, &buf)?;
        self.layers.push(layer_desc);
        Ok(())
    }

    pub fn into_inner(mut self) -> anyhow::Result<W> {
        self.finish()?;
        Ok(self.builder.take().unwrap().into_inner()?)
    }

    fn finish(&mut self) -> anyhow::Result<()> {
        let image_manifest = ImageManifestBuilder::default()
            .schema_version(SCHEMA_VERSION)
            .config(
                self.config
                    .take()
                    .context("ImageConfiguration is not set")?,
            )
            .layers(std::mem::take(&mut self.layers))
            .build()?;
        let mut buf = Vec::new();
        image_manifest.to_writer(&mut buf)?;
        let mut image_manifest_desc = self.save_blob(MediaType::ImageManifest, &buf)?;
        image_manifest_desc.set_annotations(Some(HashMap::from([(
            "org.opencontainers.image.ref.name".to_string(),
            self.get_name(),
        )])));

        let index = ImageIndexBuilder::default()
            .schema_version(SCHEMA_VERSION)
            .manifests(vec![image_manifest_desc])
            .build()?;
        let mut index_json = Vec::new();
        index.to_writer(&mut index_json)?;
        let index_json = String::from_utf8(index_json)?;
        self.save_file(Path::new("index.json"), &index_json)?;

        let version = r#"{"imageLayoutVersion":"1.0.0"}"#;
        self.save_file(Path::new("oci-layout"), version)?;

        self.builder
            .as_mut()
            .expect("builder never becomes None except on Drop")
            .finish()?;

        Ok(())
    }

    fn save_blob(&mut self, media_type: MediaType, buf: &[u8]) -> anyhow::Result<Descriptor> {
        let digest = Digest::from_buf_sha256(buf);

        let mut header = create_header(buf.len());
        self.builder
            .as_mut()
            .expect("builder never becomes None except on Drop")
            .append_data(&mut header, digest.as_path(), buf)
            .context("IO error while writing tar achive")?;

        Ok(DescriptorBuilder::default()
            .media_type(media_type)
            .size(i64::try_from(buf.len())?)
            .digest(format!("{}", digest))
            .build()
            .expect("Requirement for descriptor is mediaType, digest, and size."))
    }

    fn save_file(&mut self, dest: &Path, input: &str) -> anyhow::Result<()> {
        let mut header = create_header(input.len());
        self.builder
            .as_mut()
            .expect("builder never becomes None except on Drop")
            .append_data(&mut header, dest, input.as_bytes())
            .context("IO error while writing tar achive")?;
        Ok(())
    }
}

impl<W: io::Write> Drop for Builder<W> {
    fn drop(&mut self) {
        if self.builder.is_some() {
            let _ = self.finish();
        }
    }
}

fn now_mtime() -> u64 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}

fn create_header(size: usize) -> tar::Header {
    let mut header = tar::Header::new_gnu();
    header.set_size(u64::try_from(size).unwrap());
    header.set_cksum();
    header.set_mode(0b110100100); // rw-r--r--
    header.set_mtime(now_mtime());
    header
}

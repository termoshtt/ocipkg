//! Compose directory as a container tar

use chrono::{DateTime, Utc};
use flate2::{write::GzEncoder, Compression};
use oci_spec::image::*;
use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
};

use crate::{
    digest::Digest,
    error::*,
    image::{annotations::flat::Annotations, Config},
    media_types::{self, config_json},
    ImageName,
};

/// Build a container in oci-archive format based
/// on the [OCI image spec](https://github.com/opencontainers/image-spec)
pub struct Builder<W: io::Write> {
    /// Include a flag to check if finished
    builder: Option<tar::Builder<W>>,
    name: Option<ImageName>,
    created: Option<DateTime<Utc>>,
    author: Option<String>,
    annotations: Option<Annotations>,
    layers: Vec<Descriptor>,
    config: Config,
}

impl<W: io::Write> Builder<W> {
    pub fn new(writer: W) -> Self {
        Builder {
            builder: Some(tar::Builder::new(writer)),
            name: None,
            created: None,
            author: None,
            annotations: None,
            layers: Vec::new(),
            config: Config::default(),
        }
    }

    /// Set name of container, used in `org.opencontainers.image.ref.name` tag.
    ///
    /// If not set, a random name using UUID v4 hyphenated is set.
    pub fn set_name(&mut self, name: &ImageName) {
        self.name = Some(name.clone());
    }

    /// Set created date time in UTC
    pub fn set_created(&mut self, created: DateTime<Utc>) {
        self.created = Some(created);
    }

    /// Set additional annotations
    pub fn set_annotations(&mut self, annotations: Annotations) {
        self.annotations = Some(annotations);
    }

    /// Set the name and/or email address of the person
    /// or entity which created and is responsible for maintaining the image.
    pub fn set_author(&mut self, author: &str) {
        self.author = Some(author.to_string());
    }

    /// Append a files as a layer
    pub fn append_files(&mut self, ps: &[impl AsRef<Path>]) -> Result<()> {
        let mut ar = tar::Builder::new(GzEncoder::new(Vec::new(), Compression::default()));
        let mut files = Vec::new();
        for path in ps {
            let path = path.as_ref();
            if !path.is_file() {
                return Err(Error::NotAFile(path.to_owned()));
            }
            let name = path
                .file_name()
                .expect("This never fails since checked above")
                .to_str()
                .expect("Non-UTF8 file name");
            let mut f = fs::File::open(path)?;
            files.push(PathBuf::from(name));
            ar.append_file(name, &mut f)?;
        }
        let buf = ar.into_inner()?.finish()?;
        let layer_desc = self.save_blob(media_types::layer_tar_gzip(), &buf)?;
        self.config
            .add_layer(Digest::new(layer_desc.digest())?, files);
        self.layers.push(layer_desc);
        Ok(())
    }

    /// Append directory as a layer
    pub fn append_dir_all(&mut self, path: &Path) -> Result<()> {
        if !path.is_dir() {
            return Err(Error::NotADirectory(path.to_owned()));
        }
        let paths = fs::read_dir(path)?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .collect();

        let mut ar = tar::Builder::new(GzEncoder::new(Vec::new(), Compression::default()));
        ar.append_dir_all("", path)?;
        let buf = ar.into_inner()?.finish()?;
        let layer_desc = self.save_blob(media_types::layer_tar_gzip(), &buf)?;
        self.config
            .add_layer(Digest::new(layer_desc.digest())?, paths);
        self.layers.push(layer_desc);
        Ok(())
    }

    pub fn into_inner(mut self) -> Result<W> {
        self.finish()?;
        Ok(self.builder.take().unwrap().into_inner()?)
    }

    fn create_annotations_as_map(&self) -> HashMap<String, String> {
        if let Some(mut a) = self.annotations.clone() {
            if self.created.is_some() && a.created.is_none() {
                a.created = self.created.as_ref().map(|date| date.to_string());
            }
            if self.author.is_some() && a.authors.is_none() {
                a.authors.clone_from(&self.author);
            }
            a.to_map()
        } else {
            HashMap::new()
        }
    }

    fn finish(&mut self) -> Result<()> {
        let config = self.save_blob(config_json(), self.config.to_json()?.as_bytes())?;
        let mut builder = ImageManifestBuilder::default()
            .schema_version(SCHEMA_VERSION)
            .config(config)
            .layers(std::mem::take(&mut self.layers))
            .artifact_type(media_types::artifact());
        if self.annotations.is_some() {
            builder = builder.annotations(self.create_annotations_as_map());
        }
        let image_manifest = builder.build()?;
        let mut buf = Vec::new();
        image_manifest.to_writer(&mut buf)?;
        let mut image_manifest_desc = self.save_blob(MediaType::ImageManifest, &buf)?;

        // https://github.com/opencontainers/image-spec/blob/main/annotations.md#pre-defined-annotation-keys
        // > SHOULD only be considered valid when on descriptors on index.json within image layout.
        //
        // We need to set `org.opencontainers.image.ref.name` to index.json
        image_manifest_desc.set_annotations(Some(HashMap::from([(
            "org.opencontainers.image.ref.name".to_string(),
            self.name.clone().unwrap_or_default().to_string(),
        )])));

        let index = ImageIndexBuilder::default()
            .schema_version(SCHEMA_VERSION)
            .manifests(vec![image_manifest_desc])
            .build()?;
        let mut index_json = Vec::new();
        index.to_writer(&mut index_json)?;
        let index_json = String::from_utf8(index_json).expect("ImageIndex must returns valid JSON");
        self.save_file(Path::new("index.json"), &index_json)?;

        let version = r#"{"imageLayoutVersion":"1.0.0"}"#;
        self.save_file(Path::new("oci-layout"), version)?;

        self.builder
            .as_mut()
            .expect("builder never becomes None except on Drop")
            .finish()?;

        Ok(())
    }

    fn save_blob(&mut self, media_type: MediaType, buf: &[u8]) -> Result<Descriptor> {
        let digest = Digest::from_buf_sha256(buf);

        let mut header = create_header(buf.len());
        self.builder
            .as_mut()
            .expect("builder never becomes None except on Drop")
            .append_data(&mut header, digest.as_path(), buf)?;

        Ok(DescriptorBuilder::default()
            .media_type(media_type)
            .size(buf.len() as i64)
            .digest(format!("{}", digest))
            .build()
            .expect("Requirement for descriptor is mediaType, digest, and size."))
    }

    fn save_file(&mut self, dest: &Path, input: &str) -> Result<()> {
        let mut header = create_header(input.len());
        self.builder
            .as_mut()
            .expect("builder never becomes None except on Drop")
            .append_data(&mut header, dest, input.as_bytes())?;
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

fn create_header(size: usize) -> tar::Header {
    let mut header = tar::Header::new_gnu();
    header.set_size(u64::try_from(size).unwrap());
    header.set_cksum();
    header.set_mode(0b110100100); // rw-r--r--
    header.set_mtime(Utc::now().timestamp() as u64);
    header
}

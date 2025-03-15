//! Executable container

use super::OciArchiveBuilder;
use crate::{image::ImageBuilder, ImageName};
use anyhow::{bail, ensure, Context, Result};
use goblin::elf::Elf;
use oci_spec::image::{
    Arch, ConfigBuilder, DescriptorBuilder, ImageConfigurationBuilder, ImageManifestBuilder, Os,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Build [`Runnable`], executable container
pub struct RunnableBuilder<LayoutBuilder: ImageBuilder> {
    manifest: ImageManifestBuilder,
    entrypoint: Vec<String>,
    layout: LayoutBuilder,
    layers: Vec<oci_spec::image::Descriptor>,
    arch: Option<Arch>,
    os: Option<Os>,
}

impl<LayoutBuilder: ImageBuilder> RunnableBuilder<LayoutBuilder> {
    pub fn new(builder: LayoutBuilder) -> Result<Self> {
        Ok(Self {
            layout: builder,
            manifest: ImageManifestBuilder::default().schema_version(2_u32),
            entrypoint: Vec::new(),
            layers: Vec::new(),
            arch: None,
            os: None,
        })
    }

    pub fn append_executable(&mut self, path: &PathBuf) -> Result<()> {
        if !path.is_file() {
            anyhow::bail!("File does not exist: {:?}", path);
        }
        if !self.layers.is_empty() {
            anyhow::bail!("Only one executable is allowed");
        }

        let (arch, os) = parse_elf_header(path)?;
        self.arch = Some(arch);
        self.os = Some(os);

        let filename = path
            .file_name()
            .unwrap() // Checked above
            .to_str()
            .expect("Non-UTF8 filename");

        let mut buf = Vec::new();
        {
            let mut tar_builder = tar::Builder::new(&mut buf);
            let mut file = std::fs::File::open(path)?;
            tar_builder.append_file(filename, &mut file)?;
        }

        let (digest, size) = self.layout.add_blob(&buf)?;

        let layer_desc = DescriptorBuilder::default()
            .media_type(oci_spec::image::MediaType::ImageLayer)
            .size(size)
            .digest(digest)
            .build()?;
        self.layers.push(layer_desc);

        self.entrypoint.push(format!("/{filename}"));

        Ok(())
    }

    pub fn build(mut self) -> Result<Runnable<LayoutBuilder::Image>> {
        ensure!(
            !self.layers.is_empty() && !self.entrypoint.is_empty(),
            "No executable provided. Use `append_executable` to add one"
        );

        let cfg = ImageConfigurationBuilder::default()
            .architecture(self.arch.unwrap())
            .os(self.os.unwrap())
            .config(
                ConfigBuilder::default()
                    .entrypoint(self.entrypoint)
                    .working_dir("/")
                    .build()?,
            )
            .build()?;
        let (digest, size) = self
            .layout
            .add_blob(serde_json::to_string(&cfg)?.as_bytes())?;
        let cfg_desc = DescriptorBuilder::default()
            .media_type(oci_spec::image::MediaType::ImageConfig)
            .size(size)
            .digest(digest)
            .build()?;

        let manifest = self.manifest.config(cfg_desc).layers(self.layers).build()?;
        Ok(Runnable(self.layout.build(manifest)?))
    }
}

impl RunnableBuilder<OciArchiveBuilder> {
    pub fn new_archive_unnamed(path: PathBuf) -> Result<Self> {
        let layout = OciArchiveBuilder::new_unnamed(path)?;
        Self::new(layout)
    }

    pub fn new_archive(path: PathBuf, image_name: ImageName) -> Result<Self> {
        let layout = OciArchiveBuilder::new(path, image_name)?;
        Self::new(layout)
    }
}

/// Runnable container containing single, statically linked executable
pub struct Runnable<Layout>(Layout);

fn parse_elf_header(path: &Path) -> Result<(Arch, Os)> {
    let buffer = fs::read(path)?;
    let elf = Elf::parse(&buffer).context("Cannot parse as an ELF file")?;

    // Statically linked executable does not have interpreter (`PT_INTERP`).
    // https://refspecs.linuxbase.org/elf/gabi4+/ch5.dynamic.html#interpreter
    if elf.interpreter.is_some() {
        bail!(
            "Dynamically linked ELF executables are not supported: {}",
            path.display()
        );
    }

    let header = elf.header;
    let arch = match header.e_machine {
        goblin::elf::header::EM_X86_64 => Arch::Amd64,
        goblin::elf::header::EM_AARCH64 => Arch::ARM64,
        _ => bail!(
            "Unsupported ELF architecture: {} (Expected x86_64({}) or aarch64({})",
            header.e_machine,
            goblin::elf::header::EM_X86_64,
            goblin::elf::header::EM_AARCH64
        ),
    };
    let osabi = header.e_ident[goblin::elf::header::EI_OSABI];
    let os = match osabi {
        // XXX: OCI spec says `os` field in `application/vnd.oci.image.config.v1+json` should be GOOS value,
        //      https://github.com/opencontainers/image-spec/blob/main/config.md
        //      but it is unclear how `ELFOSABI_NONE=0` should be mapped to GOOS since it does not contains `None` variant.
        //      For now, we simply map to `Linux` because the most common use case is Linux.
        goblin::elf::header::ELFOSABI_NONE | goblin::elf::header::ELFOSABI_LINUX => Os::Linux,
        _ => bail!(
            "Unsupported ELF OS ABI: {osabi} (Expected None({}) or Linux({})",
            goblin::elf::header::ELFOSABI_NONE,
            goblin::elf::header::ELFOSABI_LINUX
        ),
    };
    Ok((arch, os))
}

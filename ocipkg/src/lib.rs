//! ocipkg consists of executables (`ocipkg` and `cargo-ocipkg`) and this crate for handling OCI Artifact without container runtime.
//!
//! See [README.md at GitHub](https://github.com/termoshtt/ocipkg) for usage of the executables.
//! This reference describes the crate part.
//!
//! Layout, Manifest, and Artifact
//! -------------------------------
//! ocipkg defines original type of OCI Artifact, `application/vnd.ocipkg.v1.artifact` to exchange directories containing static libraries.
//! But this crate is also a general purpose library to handle any kind of OCI Artifact which can contain any kind of data.
//!
//! Note that current implementation (from ocipkg 0.3.0) is based on [OCI Image specification] 1.1.0.
//!
//! ### Image Manifest
//! Every contents in a container are stored as blob, and identified by its hash digest (usually SHA256 is used).
//! [OCI Image Manifest] describes how these blobs are combined to form a container.
//! From [OCI Image specification] 1.1.0, [OCI Image Manifest] can store OCI Artifact in addition to usual executable containers.
//!
//! In this crate, [oci_spec::image::ImageManifest] is used to represent [OCI Image Manifest].
//!
//! ### Image Layout
//! [OCI Image Layout] specifies how blobs are stored as a directory.
//! Blobs are stored in `blobs/sha256/` directory with its hash digest as a file name.
//! [OCI Image Manifest] is a JSON string and also stored as a blob, thus we have to find the blob storing the manifest first.
//! `index.json` lists the digests of manifests stored in the layout.
//! There is also a file `oci-layout` which contains version of the layout itself.
//!
//! Two types of layout formats are supported. `oci-dir` format is a directory containing blobs as [OCI Image Layout] format:
//!
//! ```text
//! {dirname}/
//! ├── blobs/
//! │   └── sha256/
//! │       ├── 0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
//! │       └── ...
//! ├── index.json
//! └── oci-layout
//! ```
//!
//! `oci-archive` format is a tar archive of a `oci-dir` format. Creating a new image layout takes following steps:
//!
//! 1. Create an empty layout
//! 2. Add blobs, and store their digests
//! 3. Create a manifest using blob digests, store the manifest itself as blob, and store its digest in `index.json`
//!
//! This process is abstracted by [image::ImageLayoutBuilder]. This yields a layout which implements [image::ImageLayout] trait.
//!
//! [OCI Image specification]: https://github.com/opencontainers/image-spec/blob/v1.1.0/spec.md
//! [OCI Image Manifest]: https://github.com/opencontainers/image-spec/blob/v1.1.0/manifest.md
//! [OCI Image Layout]: https://github.com/opencontainers/image-spec/blob/v1.1.0/image-layout.md

/// Re-export since this crate exposes types in `oci_spec` crate.
pub extern crate oci_spec;

pub mod distribution;
pub mod image;
pub mod local;
pub mod media_types;

mod digest;
mod image_name;

pub use digest::Digest;
pub use image_name::ImageName;

use anyhow::Result;
use std::fs;

const STATIC_PREFIX: &str = if cfg!(target_os = "windows") {
    ""
} else {
    "lib"
};

const STATIC_EXTENSION: &str = if cfg!(target_os = "windows") {
    "lib"
} else {
    "a"
};

/// Get and link package in `build.rs` with [cargo link instructions](https://doc.rust-lang.org/cargo/reference/build-scripts.html#outputs-of-the-build-script).
///
/// This is aimed to use in [build script](https://doc.rust-lang.org/cargo/reference/build-scripts.html) a.k.a. `build.rs`.
pub fn link_package(image_name: &str) -> Result<()> {
    let image_name = ImageName::parse(image_name)?;
    let dir = local::image_dir(&image_name)?;
    if !dir.exists() {
        distribution::get_image(&image_name, false)?;
    }
    println!("cargo:rustc-link-search={}", dir.display());
    for path in fs::read_dir(&dir)?.filter_map(|entry| {
        let path = entry.ok()?.path();
        path.is_file().then_some(path)
    }) {
        let name = path
            .file_stem()
            .unwrap()
            .to_str()
            .expect("Non UTF-8 is not supported");
        let name = if let Some(name) = name.strip_prefix(STATIC_PREFIX) {
            name
        } else {
            continue;
        };
        if let Some(ext) = path.extension() {
            if ext == STATIC_EXTENSION {
                println!("cargo:rustc-link-lib=static={}", name);
            }
        }
    }
    println!("cargo:rerun-if-changed={}", dir.display());
    println!("cargo:rerun-if-env-changed=XDG_DATA_HOME");
    Ok(())
}

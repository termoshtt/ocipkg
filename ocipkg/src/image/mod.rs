//! Read and Write images based on [OCI image specification](https://github.com/opencontainers/image-spec)

pub mod annotations;

mod artifact;
mod config;
mod layout;
mod oci_archive;
mod read;
mod write;

pub use artifact::*;
pub use config::*;
pub use layout::*;
pub use oci_archive::*;
pub use read::*;
pub use write::*;

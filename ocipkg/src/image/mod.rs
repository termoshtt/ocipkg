//! Read and Write images based on [OCI image specification](https://github.com/opencontainers/image-spec)

pub mod annotations;

mod artifact;
mod layout;
mod oci_archive;
mod oci_dir;
mod platform;
mod read;
mod write;

pub use artifact::*;
pub use layout::*;
pub use oci_archive::*;
pub use oci_dir::*;
pub use platform::*;
pub use read::*;
pub use write::*;

//! Read and Write images based on [OCI image specification](https://github.com/opencontainers/image-spec)

pub mod annotations;

mod read;
mod write;

pub use read::*;
pub use write::*;

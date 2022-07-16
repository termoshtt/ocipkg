//! Read and Write images based on [OCI image specification](https://github.com/opencontainers/image-spec)

mod annotations;
mod platform;
mod read;
mod write;

pub use annotations::*;
pub use platform::*;
pub use read::*;
pub use write::*;

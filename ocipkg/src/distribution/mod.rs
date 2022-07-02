//! Rust binding to [OCI distribution spec](https://github.com/opencontainers/distribution-spec)

mod client;
mod name;
mod reference;

pub use client::*;
pub use name::Name;
pub use reference::Reference;

//! Read and Write ocipkg artifacts defined as `application/vnd.ocipkg.v1.artifact`
//!
//! See the crate level documentation for more information.

pub mod annotations;

mod config;
mod layout;
mod oci_archive;
mod oci_artifact;
mod oci_dir;
mod read;
mod remote;
mod write;

pub use config::*;
pub use layout::*;
pub use oci_archive::*;
pub use oci_artifact::*;
pub use oci_dir::*;
pub use read::*;
pub use remote::*;
pub use write::*;

//! ocipkg
//! =======

pub mod config;
pub mod distribution;
pub mod error;
pub mod image;

mod name;
mod reference;

pub use name::Name;
pub use reference::Reference;

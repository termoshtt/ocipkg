use thiserror::*;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum Error {
    #[error("Invalid digest string: {}", digest)]
    InvalidDigest { digest: String },
}

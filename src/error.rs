#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("No valid home directory path could be retrieved from the operating system.")]
    NoValidHomeDirecotry,

    #[error("Invalid digest: {0}")]
    InvalidDigest(String),

    #[error("Invalid name for repository: {0}")]
    InvalidName(String),

    #[error(transparent)]
    InvalidPort(#[from] std::num::ParseIntError),

    #[error("Invalid reference to image: {0}")]
    InvalidReference(String),

    #[error(transparent)]
    InvalidUrl(#[from] url::ParseError),
}

pub type Result<T> = std::result::Result<T, Error>;

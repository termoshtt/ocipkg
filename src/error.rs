#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("No valid home directory path could be retrieved from the operating system.")]
    NoValidHomeDirecotry,

    #[error("Invalid digest: {0}")]
    InvalidDigest(String),

    #[error("Invalid name for repository: {0}")]
    InvalidName(String),

    #[error("Invalid reference to image: {0}")]
    InvalidReference(String),
}

pub type Result<T> = std::result::Result<T, Error>;

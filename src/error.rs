#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("No valid home directory path could be retrieved from the operating system.")]
    NoValidHomeDirecotry,
}

pub type Result<T> = std::result::Result<T, Error>;

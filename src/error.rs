use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error<'a> {
    #[error("Invalid <name> of repository: {0}")]
    InvalidRepositoryName(&'a str),

    #[error("Invalid reference: {0}")]
    InvalidReference(&'a str),

    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),

    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
}

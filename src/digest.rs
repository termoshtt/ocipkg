use regex::Regex;

use crate::error::Error;

/// Digest of contents
///
/// Digest is defined in [OCI image spec](https://github.com/opencontainers/image-spec/blob/v1.0.1/descriptor.md#digests)
/// as a string satisfies following EBNF:
///
/// ```text
/// digest                ::= algorithm ":" encoded
/// algorithm             ::= algorithm-component (algorithm-separator algorithm-component)*
/// algorithm-component   ::= [a-z0-9]+
/// algorithm-separator   ::= [+._-]
/// encoded               ::= [a-zA-Z0-9=_-]+
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Digest<'a> {
    pub algorithm: &'a str,
    pub encoded: &'a str,
}

lazy_static::lazy_static! {
    static ref ENCODED_RE: Regex = Regex::new(r"[a-zA-Z0-9=_-]+").unwrap();
}

impl<'a> Digest<'a> {
    pub fn new(input: &'a str) -> Result<Self, Error<'a>> {
        let mut iter = input.split(':');
        match (iter.next(), iter.next(), iter.next()) {
            (Some(algorithm), Some(encoded), None) => {
                // FIXME: check algorithm part
                if ENCODED_RE.is_match(encoded) {
                    Ok(Digest { algorithm, encoded })
                } else {
                    Err(Error::InvalidDigest(input))
                }
            }
            _ => Err(Error::InvalidDigest(input)),
        }
    }

    /// As a fraction of path used in blobs
    pub fn as_path_fraction(&self) -> String {
        format!("{}/{}", self.algorithm, self.encoded)
    }
}

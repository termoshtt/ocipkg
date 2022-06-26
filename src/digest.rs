use regex::Regex;
use std::path::PathBuf;

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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Digest {
    pub algorithm: String,
    pub encoded: String,
}

lazy_static::lazy_static! {
    static ref ENCODED_RE: Regex = Regex::new(r"[a-zA-Z0-9=_-]+").unwrap();
}

impl Digest {
    pub fn new(input: &str) -> anyhow::Result<Self> {
        let mut iter = input.split(':');
        match (iter.next(), iter.next(), iter.next()) {
            (Some(algorithm), Some(encoded), None) => {
                // FIXME: check algorithm part
                if ENCODED_RE.is_match(encoded) {
                    Ok(Digest {
                        algorithm: algorithm.to_string(),
                        encoded: encoded.to_string(),
                    })
                } else {
                    anyhow::bail!("Invalid digest: {}", input);
                }
            }
            _ => anyhow::bail!("Invalid digest: {}", input),
        }
    }

    /// As a path used in oci-archive
    pub fn as_path(&self) -> PathBuf {
        PathBuf::from(format!("blobs/{}/{}", self.algorithm, self.encoded))
    }
}

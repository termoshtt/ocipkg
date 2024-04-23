use crate::error::*;
use oci_spec::image::Descriptor;
use regex::Regex;
use sha2::{Digest as _, Sha256};
use std::{fmt, io, path::PathBuf};

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

impl fmt::Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.algorithm, self.encoded)
    }
}

impl Digest {
    pub fn new(input: &str) -> Result<Self> {
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
                    Err(Error::InvalidDigest(input.to_string()))
                }
            }
            _ => Err(Error::InvalidDigest(input.to_string())),
        }
    }

    pub fn from_descriptor(descriptor: &Descriptor) -> Result<Self> {
        Self::new(descriptor.digest())
    }

    /// As a path used in oci-archive
    pub fn as_path(&self) -> PathBuf {
        PathBuf::from(format!("blobs/{}/{}", self.algorithm, self.encoded))
    }

    /// Calc digest using SHA-256 algorithm
    pub fn from_buf_sha256(buf: &[u8]) -> Self {
        let hash = Sha256::digest(buf);
        let digest = base16ct::lower::encode_string(&hash);
        Self {
            algorithm: "sha256".to_string(),
            encoded: digest,
        }
    }
}

/// Wrapper for calculating hash
pub struct DigestBuf<W: io::Write> {
    inner: W,
    hasher: Sha256,
}

impl<W: io::Write> DigestBuf<W> {
    pub fn new(inner: W) -> Self {
        DigestBuf {
            inner,
            hasher: Sha256::new(),
        }
    }

    pub fn finish(self) -> (W, Digest) {
        let hash = self.hasher.finalize();
        let digest = base16ct::lower::encode_string(&hash);
        (
            self.inner,
            Digest {
                algorithm: "sha256".to_string(),
                encoded: digest,
            },
        )
    }
}

impl<W: io::Write> io::Write for DigestBuf<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.hasher.update(buf);
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

use anyhow::{bail, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use std::{fmt, path::PathBuf, str::FromStr};

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

impl From<oci_spec::image::Digest> for Digest {
    fn from(digest: oci_spec::image::Digest) -> Self {
        Digest {
            algorithm: digest.algorithm().to_string(),
            encoded: digest.digest().to_string(),
        }
    }
}

impl TryFrom<&Digest> for oci_spec::image::Digest {
    type Error = anyhow::Error;
    fn try_from(digest: &Digest) -> Result<Self> {
        Ok(oci_spec::image::Digest::from_str(&digest.to_string())?)
    }
}

lazy_static::lazy_static! {
    static ref ENCODED_RE: Regex = Regex::new(r"[a-zA-Z0-9=_-]+").unwrap();
}

impl fmt::Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.algorithm, self.encoded)
    }
}

impl Serialize for Digest {
    fn serialize<S>(&self, serializer: S) -> std::prelude::v1::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Digest {
    fn deserialize<D>(deserializer: D) -> std::prelude::v1::Result<Digest, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Digest::new(&s).map_err(serde::de::Error::custom)
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
                    bail!("Invalid digest: {}", input);
                }
            }
            _ => bail!("Invalid digest: {}", input),
        }
    }

    pub fn from_descriptor(descriptor: &oci_spec::image::Descriptor) -> Result<Self> {
        Self::new(descriptor.digest().as_ref())
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

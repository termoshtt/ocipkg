use crate::error;
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::{collections::HashMap, fmt, str::FromStr};

/// OCI Image Index Specification, `index.json` file in oci-dir format.
///
/// https://github.com/opencontainers/image-spec/blob/master/image-index.md
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Index {
    schema_version: u32,
    media_type: Option<String>,
    manifests: Vec<Manifest>,
    annotations: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    media_type: String,
    size: usize,
    digest: Digest,
    platform: Option<Platform>,
}

#[derive(Debug)]
pub enum Digest {
    SHA256(String),
}

impl FromStr for Digest {
    type Err = error::Error;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if let Some(digest) = input.strip_prefix("sha256:") {
            Ok(Digest::SHA256(digest.to_string()))
        } else {
            Err(error::Error::InvalidDigest {
                digest: input.to_string(),
            })
        }
    }
}

/// Implement custom Serializer for Digest
///
/// See https://serde.rs/impl-serialize.html
impl Serialize for Digest {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Digest::SHA256(digest) => serializer.serialize_str(&format!("sha256:{}", digest)),
        }
    }
}

/// Implement custom Deserializer for Digest
///
/// See https://serde.rs/impl-deserialize.html for detail
struct DigestVisitor;
impl<'de> Visitor<'de> for DigestVisitor {
    type Value = Digest;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("SHA256 hash string with 'sha256:' prefix")
    }
    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
        match Digest::from_str(v) {
            Ok(digest) => Ok(digest),
            Err(e) => Err(E::custom(e)),
        }
    }
}
impl<'de> Deserialize<'de> for Digest {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Digest, D::Error> {
        deserializer.deserialize_str(DigestVisitor)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Platform {
    architecture: String,
    os: String,
}

/// oci-layout file
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Layout {
    image_layout_version: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digest() {
        let digest = Digest::from_str(
            "sha256:e692418e4cbaf90ca69d05a66403747baa33ee08806650b51fab815ad7fc331f",
        )
        .unwrap();
        dbg!(digest);
    }

    #[test]
    fn index() {
        let example = r#"
        {
          "schemaVersion": 2,
          "manifests": [
            {
              "mediaType": "application/vnd.oci.image.manifest.v1+json",
              "size": 7143,
              "digest": "sha256:e692418e4cbaf90ca69d05a66403747baa33ee08806650b51fab815ad7fc331f",
              "platform": {
                "architecture": "ppc64le",
                "os": "linux"
              }
            },
            {
              "mediaType": "application/vnd.oci.image.manifest.v1+json",
              "size": 7682,
              "digest": "sha256:5b0bcabd1ed22e9fb1310cf6c2dec7cdef19f0ad69efa1f392e94a4333501270",
              "platform": {
                "architecture": "amd64",
                "os": "linux"
              }
            }
          ],
          "annotations": {
            "com.example.key1": "value1",
            "com.example.key2": "value2"
          }
        }
        "#;
        let image_index: Index = serde_json::from_str(example).unwrap();
        dbg!(image_index);
    }

    #[test]
    fn layout() {
        let image_layout: Layout =
            serde_json::from_str(r#"{ "imageLayoutVersion": "1.0.0" }"#).unwrap();
        dbg!(&image_layout);
        assert_eq!(image_layout.image_layout_version, "1.0.0");
    }
}

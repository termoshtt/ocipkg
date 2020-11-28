//! cync

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OCI Image Index Specification, `index.json` file in oci-dir format.
///
/// https://github.com/opencontainers/image-spec/blob/master/image-index.md
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Index {
    schema_version: u32,
    media_type: Option<String>,
    manifests: Vec<Manifest>,
    annotations: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    media_type: String,
    platform: Option<Platform>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Platform {
    architecture: String,
    os: String,
}

/// oci-layout file
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Layout {
    image_layout_version: String,
}

#[cfg(test)]
mod tests {
    use super::*;

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

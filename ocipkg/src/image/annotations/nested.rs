//! Annotations with nested serialization/deserialization

use crate::error::*;
use serde::{Deserialize, Serialize};

/// Root namespace for annotations
///
/// See [Annotations] document.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Root {
    org: Org,
}

/// `org.*` annotations
///
/// See [Annotations] document.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Org {
    pub opencontainers: OpenContainers,
}

/// `org.opencontainers.*` annotations
///
/// See [Annotations] document.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct OpenContainers {
    pub image: Annotations,
}

/// `org.opencontainers.image.*` annotations
///
/// See [Pre-Defined Annotation Keys](https://github.com/opencontainers/image-spec/blob/main/annotations.md#pre-defined-annotation-keys)
/// in OCI image spec.
///
/// This is designed to use with TOML:
///
/// ```
/// use ocipkg::image::annotations::nested::*;
///
/// // Read TOML
/// let a = Annotations::from_toml(
///     r#"
///     [org.opencontainers.image]
///     url = "https://github.com/termoshtt/ocipkg"
///     "#,
/// )
/// .unwrap();
/// assert_eq!(
///     a,
///     Annotations {
///         url: Some("https://github.com/termoshtt/ocipkg".to_string()),
///         ..Default::default()
///     }
/// );
///
/// // Dump to TOML
/// let a = Annotations {
///     url: Some("https://github.com/termoshtt/ocipkg".to_string()),
///     ..Default::default()
/// };
/// assert_eq!(
///     a.to_toml().trim(),
///     r#"
/// [org.opencontainers.image]
/// url = "https://github.com/termoshtt/ocipkg"
///     "#.trim()
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Annotations {
    /// `org.opencontainers.image.created`
    ///
    /// date and time on which the image was built (string, date-time as defined by RFC 3339).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,

    /// `org.opencontainers.image.authors`
    ///
    /// contact details of the people or organization responsible for the image (freeform string)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authors: Option<String>,

    /// `org.opencontainers.image.url`
    ///
    /// URL to find more information on the image (string)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// `org.opencontainers.image.documentation`
    ///
    /// URL to get documentation on the image (string)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,

    /// `org.opencontainers.image.source`
    ///
    /// URL to get source code for building the image (string)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// `org.opencontainers.image.version`
    ///
    /// version of the packaged software
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// `org.opencontainers.image.revision`
    ///
    /// Source control revision identifier for the packaged software.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,

    /// `org.opencontainers.image.vendor`
    ///
    /// Name of the distributing entity, organization or individual.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,

    /// `org.opencontainers.image.licenses`
    ///
    /// License(s) under which contained software is distributed as an SPDX License Expression.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub licenses: Option<String>,

    /// `org.opencontainers.image.title`
    ///
    /// Human-readable title of the image (string)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// `org.opencontainers.image.description`
    ///
    /// Human-readable description of the software packaged in the image (string)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// `org.opencontainers.image.ref.*` components
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#ref: Option<Ref>,

    /// `org.opencontainers.image.base.*` components
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base: Option<Base>,
}

/// `org.opencontainers.image.base.*` annotations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Base {
    /// `org.opencontainers.image.base.digest`
    ///
    /// Digest of the image this image is based on (string)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,

    /// `org.opencontainers.image.base.name`
    ///
    /// Annotations reference of the image this image is based on (string)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// `org.opencontainers.image.ref.*` annotations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Ref {
    /// `org.opencontainers.image.ref.name`
    ///
    /// Name of the reference for a target (string).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl Annotations {
    pub fn from_toml(input: &str) -> Result<Self> {
        let root: Root = toml::from_str(input)?;
        Ok(root.org.opencontainers.image)
    }

    pub fn to_toml(&self) -> String {
        let root = Root {
            org: Org {
                opencontainers: OpenContainers {
                    image: self.clone(),
                },
            },
        };
        toml::to_string_pretty(&root).unwrap()
    }
}

impl From<super::flat::Annotations> for Annotations {
    fn from(flat: super::flat::Annotations) -> Self {
        let base = if flat.base_name.is_none() && flat.base_digest.is_none() {
            None
        } else {
            Some(Base {
                name: flat.base_name,
                digest: flat.base_digest,
            })
        };
        let r#ref = flat.ref_name.map(|name| Ref { name: Some(name) });
        Annotations {
            created: flat.created,
            authors: flat.authors,
            url: flat.url,
            documentation: flat.documentation,
            description: flat.description,
            title: flat.title,
            source: flat.source,
            version: flat.version,
            revision: flat.revision,
            vendor: flat.vendor,
            licenses: flat.licenses,
            r#ref,
            base,
        }
    }
}

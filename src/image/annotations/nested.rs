//! Annotations with nested serialization/deserialization

use serde::{Deserialize, Serialize};

/// `org.*` annotations
///
/// See [Annotations] document.
#[derive(Debug, Serialize, Deserialize)]
pub struct Org {
    pub opencontainers: OpenContainers,
}

/// `org.opencontainers.*` annotations
///
/// See [Annotations] document.
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenContainers {
    pub image: Annotations,
}

/// `org.opencontainers.image.*` annotations
///
/// See [Pre-Defined Annotation Keys](https://github.com/opencontainers/image-spec/blob/main/annotations.md#pre-defined-annotation-keys)
/// in OCI image spec.
#[derive(Debug, Serialize, Deserialize)]
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

    /// `org.opencontainers.image.ref.name`
    ///
    /// Name of the reference for a target (string).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ref_name: Option<String>,

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

    /// `org.opencontainers.image.base.*` components
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base: Option<Base>,
}

/// `org.opencontainers.image.base.*` annotations
#[derive(Debug, Serialize, Deserialize)]
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

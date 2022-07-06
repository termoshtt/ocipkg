use oci_spec::image::*;

/// Annotations defined in `org.opencontainers.image.*` namespace
///
/// See [Pre-Defined Annotation Keys](https://github.com/opencontainers/image-spec/blob/main/annotations.md#pre-defined-annotation-keys)
/// in OCI image spec.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Annotations {
    /// `org.opencontainers.image.created`
    ///
    /// date and time on which the image was built (string, date-time as defined by RFC 3339).
    pub created: Option<String>,

    /// `org.opencontainers.image.authors`
    ///
    /// contact details of the people or organization responsible for the image (freeform string)
    pub authors: Option<String>,

    /// `org.opencontainers.image.url`
    ///
    /// URL to find more information on the image (string)
    pub url: Option<String>,

    /// `org.opencontainers.image.documentation`
    ///
    /// URL to get documentation on the image (string)
    pub documentation: Option<String>,

    /// `org.opencontainers.image.source`
    ///
    /// URL to get source code for building the image (string)
    pub source: Option<String>,

    /// `org.opencontainers.image.version`
    ///
    /// version of the packaged software
    pub version: Option<String>,

    /// `org.opencontainers.image.revision`
    ///
    /// Source control revision identifier for the packaged software.
    pub revision: Option<String>,

    /// `org.opencontainers.image.vendor`
    ///
    /// Name of the distributing entity, organization or individual.
    pub vendor: Option<String>,

    /// `org.opencontainers.image.licenses`
    ///
    /// License(s) under which contained software is distributed as an SPDX License Expression.
    pub licenses: Option<String>,

    /// `org.opencontainers.image.ref.name`
    ///
    /// Name of the reference for a target (string).
    pub ref_name: Option<String>,

    /// `org.opencontainers.image.title`
    ///
    /// Human-readable title of the image (string)
    pub title: Option<String>,

    /// `org.opencontainers.image.description`
    ///
    /// Human-readable description of the software packaged in the image (string)
    pub description: Option<String>,

    /// `org.opencontainers.image.base.digest`
    ///
    /// Digest of the image this image is based on (string)
    pub base_digest: Option<String>,

    /// `org.opencontainers.image.base.name`
    ///
    /// Image reference of the image this image is based on (string)
    pub base_name: Option<String>,
}

macro_rules! impl_into_iter_part {
    ($dest:expr, $tag:ident, $member:expr) => {
        if let Some(value) = $member {
            $dest.push(($tag.to_string(), value))
        }
    };
}

impl IntoIterator for Annotations {
    type Item = (String, String);
    type IntoIter = std::vec::IntoIter<(String, String)>;
    fn into_iter(self) -> Self::IntoIter {
        let mut a = Vec::new();
        impl_into_iter_part!(a, ANNOTATION_AUTHORS, self.authors);
        impl_into_iter_part!(a, ANNOTATION_BASE_IMAGE_DIGEST, self.base_digest);
        impl_into_iter_part!(a, ANNOTATION_BASE_IMAGE_NAME, self.base_name);
        impl_into_iter_part!(a, ANNOTATION_CREATED, self.created);
        impl_into_iter_part!(a, ANNOTATION_DESCRIPTION, self.description);
        impl_into_iter_part!(a, ANNOTATION_DOCUMENTATION, self.documentation);
        impl_into_iter_part!(a, ANNOTATION_LICENSES, self.licenses);
        impl_into_iter_part!(a, ANNOTATION_REF_NAME, self.ref_name);
        impl_into_iter_part!(a, ANNOTATION_REVISION, self.revision);
        impl_into_iter_part!(a, ANNOTATION_TITLE, self.title);
        impl_into_iter_part!(a, ANNOTATION_URL, self.url);
        impl_into_iter_part!(a, ANNOTATION_VENDOR, self.vendor);
        impl_into_iter_part!(a, ANNOTATION_VERSION, self.version);
        a.into_iter()
    }
}

impl<'s> std::iter::FromIterator<(&'s str, &'s str)> for Annotations {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (&'s str, &'s str)>,
    {
        let mut annotations = Self::default();
        for (key, value) in iter {
            // after-priority
            let _pre = match key {
                ANNOTATION_AUTHORS => annotations.authors.replace(value.to_string()),
                ANNOTATION_BASE_IMAGE_DIGEST => annotations.base_digest.replace(value.to_string()),
                ANNOTATION_BASE_IMAGE_NAME => annotations.base_name.replace(value.to_string()),
                ANNOTATION_CREATED => annotations.created.replace(value.to_string()),
                ANNOTATION_DESCRIPTION => annotations.description.replace(value.to_string()),
                ANNOTATION_DOCUMENTATION => annotations.documentation.replace(value.to_string()),
                ANNOTATION_LICENSES => annotations.licenses.replace(value.to_string()),
                ANNOTATION_REF_NAME => annotations.ref_name.replace(value.to_string()),
                ANNOTATION_REVISION => annotations.revision.replace(value.to_string()),
                ANNOTATION_SOURCE => annotations.source.replace(value.to_string()),
                ANNOTATION_TITLE => annotations.title.replace(value.to_string()),
                ANNOTATION_URL => annotations.url.replace(value.to_string()),
                ANNOTATION_VENDOR => annotations.vendor.replace(value.to_string()),
                ANNOTATION_VERSION => annotations.version.replace(value.to_string()),
                _ => None,
            };
        }
        annotations
    }
}

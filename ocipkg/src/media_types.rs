use oci_spec::image::MediaType;

/// The media type of "ocipkg artifact" used as `artifactType` in the OCI image manifest
pub fn artifact() -> MediaType {
    MediaType::Other("application/vnd.ocipkg.v1.artifact".to_string())
}

/// The media type used in `config` descriptor of ocipkg artifact
///
/// The content of the descriptor of this type must be a JSON of [crate::image::Config]
pub fn config_json() -> MediaType {
    MediaType::Other("application/vnd.ocipkg.v1.config+json".to_string())
}

/// The media type used in `layer` descriptor of ocipkg artifact
///
/// The content of the descriptor of this type must be a tar.gz of the layer
pub fn layer_tar_gzip() -> MediaType {
    MediaType::Other("application/vnd.ocipkg.v1.layer.tar+gzip".to_string())
}

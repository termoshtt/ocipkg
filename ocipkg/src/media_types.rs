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

/// Test media_type is imageindex
///
/// DockerV2S2 can't directly match by MediaType
pub fn is_imageindex(media_type: &str) -> bool {
    matches!(
        media_type,
        "application/vnd.docker.distribution.manifest.list.v2+json"
            | "application/vnd.oci.image.index.v1+json"
    )
}

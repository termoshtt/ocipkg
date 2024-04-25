use oci_spec::image::MediaType;

pub fn artifact() -> MediaType {
    MediaType::Other("application/vnd.ocipkg.v1.artifact".to_string())
}

pub fn directory_tar_gzip() -> MediaType {
    MediaType::Other("application/vnd.ocipkg.v1.directory.tar+gzip".to_string())
}

pub fn file_gzip() -> MediaType {
    MediaType::Other("application/vnd.ocipkg.v1.file+gzip".to_string())
}

use oci_spec::image::Digest;
use sha2::{Digest as _, Sha256};
use std::{path::PathBuf, str::FromStr};

pub trait DigestExt {
    fn eval_sha256_digest(buf: &[u8]) -> Self;
    fn as_path(&self) -> PathBuf;
}

impl DigestExt for Digest {
    fn eval_sha256_digest(buf: &[u8]) -> Self {
        let hash = Sha256::digest(buf);
        let digest = base16ct::lower::encode_string(&hash);
        oci_spec::image::Digest::from_str(&format!("sha256:{}", digest)).unwrap()
    }

    fn as_path(&self) -> PathBuf {
        PathBuf::from(format!("blobs/{}/{}", self.algorithm(), self.digest()))
    }
}

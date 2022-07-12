use oci_spec::image::*;

#[derive(Debug)]
pub struct Target {
    pub arch: Arch,
    pub os: Os,
    pub os_version: Option<String>,
    pub os_feature: Option<String>,
    pub variant: Option<String>,
}

impl Target {
    pub fn from_cargo_cfg() -> Self {
        let arch = if cfg!(x86_64) {
            Arch::Amd64
        } else {
            unreachable!()
        };
        let os = if cfg!(linux) {
            Os::Linux
        } else {
            unreachable!()
        };
        Target {
            arch,
            os,
            os_version: None,
            os_feature: None,
            variant: None,
        }
    }
}

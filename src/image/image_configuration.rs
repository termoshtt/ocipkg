use oci_spec::image::*;

pub trait PlatformEx: Sized {
    fn from_cargo_cfg() -> Self;
    fn from_target_triple() -> anyhow::Result<Self>;
}

impl PlatformEx for Platform {
    fn from_cargo_cfg() -> Self {
        let (arch, variant): (Arch, Option<String>) = if cfg!(x86_64) {
            (Arch::Amd64, None)
        } else if cfg!(aarch64) {
            (Arch::ARM64, Some("v8".to_string()))
        } else {
            // FIXME Support other CPU
            unreachable!()
        };
        let os = if cfg!(linux) {
            Os::Linux
        } else if cfg!(windows) {
            Os::Windows
        } else {
            // FIXME Support other OS
            unreachable!()
        };
        let mut builder = PlatformBuilder::default().os(os).architecture(arch);
        if let Some(variant) = variant {
            builder = builder.variant(variant);
        }
        builder.build().unwrap()
    }

    fn from_target_triple() -> anyhow::Result<Self> {
        todo!()
    }
}

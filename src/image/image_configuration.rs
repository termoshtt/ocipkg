use oci_spec::image::*;

pub trait PlatformEx: Sized {
    fn from_cargo_cfg() -> Self;
    fn from_target_triple(target_triple: &str) -> anyhow::Result<Self>;
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

    fn from_target_triple(target_triple: &str) -> anyhow::Result<Self> {
        let parts: Vec<&str> = target_triple.split('-').collect();
        let (arch, os) = match parts[..] {
            [arch, _vender, os, _env] => (arch, os),
            [arch, os, _env] => (arch, os),
            _ => anyhow::bail!("Unknown target triple: {}", target_triple),
        };
        let (arch, variant) = match arch {
            "x86_64" => (Arch::Amd64, None),
            "aarch64" => (Arch::ARM64, Some("v8".to_string())),
            _ => anyhow::bail!("Unsupported arch: {}", arch),
        };
        let os = match os {
            "linux" => Os::Linux,
            "windows" => Os::Windows,
            _ => anyhow::bail!("Unsupported OS: {}", os),
        };
        let mut builder = PlatformBuilder::default().os(os).architecture(arch);
        if let Some(variant) = variant {
            builder = builder.variant(variant);
        }
        Ok(builder.build().unwrap())
    }
}

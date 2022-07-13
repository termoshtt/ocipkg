use oci_spec::image::*;

/// Extension of [Platform]
pub trait PlatformEx: Sized {
    /// Create [Platform] using [std::cfg] macro
    fn from_cfg_macro() -> Self;

    /// Create [Platform] from target-triple.
    ///
    /// This does not support unnormalized target triple which LLVM may accept,
    /// e.g. `x86_64`, `x86_64-linux`, and so on.
    fn from_target_triple(target_triple: &str) -> anyhow::Result<Self>;
}

impl PlatformEx for Platform {
    fn from_cfg_macro() -> Self {
        let (arch, variant): (Arch, Option<String>) = if cfg!(target_arch = "x86_64") {
            (Arch::Amd64, None)
        } else if cfg!(target_arch = "i686") {
            (Arch::i386, None)
        } else if cfg!(target_arch = "aarch64") {
            (Arch::ARM64, Some("v8".to_string()))
        } else {
            unimplemented!("Unsupported CPU")
        };
        let os = if cfg!(target_os = "linux") {
            Os::Linux
        } else if cfg!(target_os = "windows") {
            Os::Windows
        } else if cfg!(target_os = "macos") {
            Os::Darwin
        } else {
            unimplemented!("Unsupported OS")
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
            "i686" => (Arch::i386, None),
            "aarch64" => (Arch::ARM64, Some("v8".to_string())),
            _ => anyhow::bail!("Unsupported arch: {}", arch),
        };
        let os = match os {
            "linux" => Os::Linux,
            "windows" => Os::Windows,
            "apple" => Os::Darwin,
            _ => anyhow::bail!("Unsupported OS: {}", os),
        };
        let mut builder = PlatformBuilder::default().os(os).architecture(arch);
        if let Some(variant) = variant {
            builder = builder.variant(variant);
        }
        Ok(builder.build().unwrap())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    fn from_cargo_cfg() {
        let platform = Platform::from_cfg_macro();
        assert_eq!(platform.architecture(), &Arch::Amd64);
        assert_eq!(platform.os(), &Os::Linux);
    }

    #[test]
    fn from_target_triple() {
        fn test(target_triple: &str, arch: &Arch, os: &Os) {
            let platform = Platform::from_target_triple(target_triple).unwrap();
            assert_eq!(platform.architecture(), arch);
            assert_eq!(platform.os(), os);
        }
        // Tier 1 targets of rustc
        // https://doc.rust-lang.org/nightly/rustc/platform-support.html
        test("aarch64-unknown-linux-gnu", &Arch::ARM64, &Os::Linux);
        test("i686-pc-windows-gnu", &Arch::i386, &Os::Windows);
        test("i686-pc-windows-msvc", &Arch::i386, &Os::Windows);
        test("i686-unknown-linux-gnu", &Arch::i386, &Os::Linux);
        test("x86_64-apple-darwin", &Arch::Amd64, &Os::Darwin);
        test("x86_64-pc-windows-gnu", &Arch::Amd64, &Os::Windows);
        test("x86_64-pc-windows-msvc", &Arch::Amd64, &Os::Windows);
        test("x86_64-unknown-linux-gnu", &Arch::Amd64, &Os::Linux);
    }
}

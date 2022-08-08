#[cfg(feature = "cli")]
use vergen::{vergen, Config};

fn main() {
    #[cfg(feature = "cli")]
    {
        let mut cfg = Config::default();
        *cfg.sysinfo_mut().name_mut() = false;
        vergen(cfg).expect("Fail to generate version info");
    }
}

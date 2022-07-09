fn main() -> anyhow::Result<()> {
    ocipkg::link_package("ghcr.io/termoshtt/ocipkg/rust-lib:latest")?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    ocipkg::link_package("localhost:5000/test_repo:tag1")?;
    Ok(())
}

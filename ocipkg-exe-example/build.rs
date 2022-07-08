fn main() -> anyhow::Result<()> {
    ocipkg::find_package("localhost:5000/test_repo:tag1")?;
    Ok(())
}

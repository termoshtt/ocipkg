use oci_spec::image::*;
use std::{
    fs,
    io::{Read, Seek},
    path::*,
};

/// Get and deserialize index.json
fn get_index(input: &mut fs::File) -> anyhow::Result<ImageIndex> {
    input.rewind()?;
    let mut ar = tar::Archive::new(input);
    // Searched linearly since tar archive does not have offset table.
    for entry in ar.entries_with_seek()? {
        let mut entry = entry?;
        if entry.path()?.as_os_str() == "index.json" {
            let mut out = Vec::new();
            entry.read_to_end(&mut out)?;
            return Ok(ImageIndex::from_reader(&*out)?);
        }
    }
    anyhow::bail!("index.json not found in oci-archive")
}

/// Look up descripters from blobs listed in index.json
fn get_manifests(input: &mut fs::File, descs: &[Descriptor]) -> anyhow::Result<Vec<ImageManifest>> {
    input.rewind()?;
    let mut ar = tar::Archive::new(input);

    let targets = descs
        .iter()
        .map(|desc| {
            let mut iter = desc.digest().split(":");
            match (iter.next(), iter.next()) {
                (Some(algorithm), Some(hash)) => Ok((algorithm, hash)),
                _ => anyhow::bail!("Invalid digest in index.json"),
            }
        })
        .collect::<Result<Vec<(&str, &str)>, _>>()?;

    let mut manifests = Vec::new();

    // Searched linearly since tar archive does not have offset table.
    for entry in ar.entries_with_seek()? {
        let entry = entry?;
        let path = entry.path()?;

        // Path of manifest must be in `blobs/{algorithm}/{hash}` form
        let mut iter = path.components();
        let (algorithm, hash) = match (iter.next(), iter.next(), iter.next()) {
            (
                Some(Component::Normal(top)),
                Some(Component::Normal(algorithm)),
                Some(Component::Normal(hash)),
            ) => {
                if top != "blobs" {
                    continue;
                }
                (algorithm, hash)
            }
            _ => continue,
        };
        if !targets.iter().any(|t| algorithm == t.0 && hash == t.1) {
            continue;
        }
        manifests.push(ImageManifest::from_reader(entry)?);
    }
    anyhow::ensure!(
        descs.len() == manifests.len(),
        "Some manifest not found in container"
    );
    Ok(manifests)
}

/// Load oci-archive into local storage
pub fn load(input: &Path) -> anyhow::Result<()> {
    if !input.exists() {
        anyhow::bail!("Input file does not exist");
    }
    let mut f = fs::File::open(input)?;
    let index = get_index(&mut f)?;
    dbg!(&index);

    let manifest = get_manifests(&mut f, index.manifests())?;
    dbg!(manifest);

    Ok(())
}

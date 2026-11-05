use anyhow::Result;
use directories::ProjectDirs;
use ocipkg::{image, local, ImageName};
use std::fs;

#[test]
fn load_rejects_traversal_before_writing_or_deleting() -> Result<()> {
    let temp = tempfile::tempdir()?;
    local::set_project_dirs(ProjectDirs::from_path(temp.path().join("store")).unwrap())?;
    let store = local::data_dir()?;
    fs::create_dir_all(&store)?;

    let payload = temp.path().join("payload.txt");
    fs::write(&payload, "archive contents")?;
    let outside = store.parent().unwrap().join("outside").join("__latest");
    assert!(outside.starts_with(temp.path()));
    assert!(!outside.starts_with(&store));

    // Public fields also let archive builders serialize a name the parser must reject.
    let mut name = ImageName::parse("localhost/outside:latest")?;
    name.hostname = "..".to_string();
    let archive = temp.path().join("traversal.tar");
    let mut builder = image::Builder::new(archive.clone(), name)?;
    builder.append_files(&[&payload])?;
    builder.build()?;

    for overwrite in [false, true] {
        assert!(image::load(&archive, overwrite).is_err());
        assert!(
            !outside.exists(),
            "created a directory outside the image store"
        );
        assert_eq!(fs::read_dir(&store)?.count(), 0);
    }

    fs::create_dir_all(&outside)?;
    let marker = outside.join("keep.txt");
    fs::write(&marker, "keep me")?;
    for overwrite in [false, true] {
        assert!(image::load(&archive, overwrite).is_err());
        assert_eq!(fs::read_to_string(&marker)?, "keep me");
        assert!(!outside.join("payload.txt").exists());
        assert!(!outside.join(".oci-dir").exists());
        assert_eq!(fs::read_dir(&store)?.count(), 0);
    }

    // Valid names retain the existing load and overwrite behavior.
    for (index, input) in [
        "localhost:5000/example/image:latest",
        "test_repo/testing:tag1",
        "test+repo/testing:tag1",
        "test@repo/testing:tag1",
        "test--repo/testing:tag1",
    ]
    .iter()
    .enumerate()
    {
        let name = ImageName::parse(input)?;
        let destination = local::image_dir(&name)?;
        assert!(destination.starts_with(&store));
        let archive = temp.path().join(format!("valid-{index}.tar"));
        let mut builder = image::Builder::new(archive.clone(), name)?;
        builder.append_files(&[&payload])?;
        builder.build()?;
        image::load(&archive, false)?;
        assert_eq!(
            fs::read_to_string(destination.join("payload.txt"))?,
            "archive contents"
        );
        assert!(image::load(&archive, false).is_err());
        fs::write(destination.join("old.txt"), "replace me")?;
        image::load(&archive, true)?;
        assert!(!destination.join("old.txt").exists());
        assert_eq!(
            fs::read_to_string(destination.join("payload.txt"))?,
            "archive contents"
        );
    }
    Ok(())
}

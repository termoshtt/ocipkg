use anyhow::Context;
use directories::ProjectDirs;
use std::path::*;

pub const PROJECT_NAME: &str = "ocipkg";

/// Project root data directory
pub fn data_dir() -> anyhow::Result<PathBuf> {
    let p = ProjectDirs::from("", PROJECT_NAME, PROJECT_NAME)
        .context("System does not provide valid $HOME path")?;
    let dir = p.data_dir();
    Ok(dir.to_owned())
}

/// Create data directory for each image
///
/// Input `name` must be in [the form of "org.opencontainers.image.ref.name"](https://github.com/opencontainers/image-spec/blob/main/annotations.md#pre-defined-annotation-keys)
///
/// ```text
/// ref       ::= component ("/" component)*
/// component ::= alphanum (separator alphanum)*
/// alphanum  ::= [A-Za-z0-9]+
/// separator ::= [-._:@+] | "--"
/// ```
///
/// Some charactors, `:` and `@` cannot be used in directory path since this crate
/// also supports [Windows with NTFS](https://docs.microsoft.com/en-us/windows/win32/fileio/naming-a-file).
/// These charactors are replaced as following:
///
/// - `:` to `__`
/// - `@` to `___`
///
/// Note that [docker allows `__`](https://docs.docker.com/engine/reference/commandline/tag/), but OCI image spec does not allow it.
///
/// This function creates directories for each components
/// and special separators `:`, `@`, and `+` to match the practice in container managements.
///
/// ```
/// std::env::set_var("XDG_DATA_HOME", "/data");
/// assert_eq!(
///   ocipkg::config::image_dir("ghcr.io/termoshtt/ocipkg:0.1.0").unwrap().as_os_str(),
///   "/data/ocipkg/ghcr.io/termoshtt/ocipkg/__0.1.0"
///   //                ^                   ^ `:` is translated to `__` and a directory is created
///   //                ^ do not created directory for `.`
/// );
/// ```
///
pub fn image_dir(name: &str) -> anyhow::Result<PathBuf> {
    Ok(data_dir()?.join(
        name.replace(':', "/__")
            .replace('@', "/___")
            .replace('+', "/+"),
    ))
}

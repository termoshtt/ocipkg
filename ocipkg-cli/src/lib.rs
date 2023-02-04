use anyhow::{bail, Result};
use fuse::{
    FileAttr, FileType, Filesystem, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry, Request,
};
use libc::ENOENT;
use ocipkg::*;
use std::{
    collections::{BTreeMap, HashMap},
    ffi::OsStr,
    path::*,
};
use time::Timespec;

/// Time to live (TTL) of filesystem cache
const TTL: Timespec = Timespec { sec: 1, nsec: 0 };
/// The UNIX epoch, `1970-01-01 00:00:00`
const UNIX_EPOCH: Timespec = Timespec { sec: 0, nsec: 0 };
/// Inode of filesystem root
const ROOT_INODE: u64 = 1;

/// Read-only filesystem corresponding to a container
///
/// ```text
/// ...
/// └─ __tag1/
///      └─ dir1/
///         └─ file1
/// ```
///
#[derive(Debug, Clone)]
struct Container {
    /// Inode of head directory, i.e. the inode of `__tag` directory.
    base_ino: u64,
    /// Cache of file paths in the container.
    paths: Vec<PathBuf>,
    /// Relative path from container root to attribute
    attrs: HashMap<PathBuf, FileAttr>,
    /// Image name
    name: ocipkg::ImageName,
}

impl Container {
    fn get_attr(&self, ino: u64) -> Option<&FileAttr> {
        if ino < self.base_ino {
            return None;
        }
        let index = (ino - self.base_ino) as usize;
        let path = self.paths.get(index)?;
        Some(&self.attrs[path])
    }
}

/// Directory structure in FUSE should be like following diagram:
///
/// ```text
/// Root (/)
///  │
///  ├─ some.registry/
///  │  └─ project_name/
///  │     └─ container_name/
///  │         └─ __tag1/
///  │            └─ dir1/
///  │               └─ file1
///  │
///  └─ another.registry__8080/
///     └─ project_name/
///        └─ __tag1/
///           └─ dir1/
///              └─ file1
/// ```
///
/// - The contents in a container will be placed under the directory
///   corresponding to the container, and managed by [Container].
///
/// - Inodes corresponding to files and directories in a container
///   must be continuous. For example, if the inode of `__tag1`
///   in above diagram is `3`, those of `dir1/` and `file1` must be
///   in `[3 + 1, 3 + {number of files}]` i.e. `4` and `5`.
///
/// - The inodes of directories corresponding to registries
///   or container namespace should not be continuous.
///
#[derive(Debug, Clone)]
pub struct OcipkgFS {
    attr: FileAttr,
    inode_count: u64,
    containers: Vec<Container>,
    /// Inode to path
    paths: BTreeMap<u64, PathBuf>,
    /// Path to attribute
    attrs: HashMap<PathBuf, FileAttr>,
}

impl OcipkgFS {
    pub fn new() -> Self {
        let attr = FileAttr {
            ino: ROOT_INODE,
            size: 0,
            blocks: 0,
            atime: UNIX_EPOCH,
            mtime: UNIX_EPOCH,
            ctime: UNIX_EPOCH,
            crtime: UNIX_EPOCH,
            kind: FileType::Directory,
            perm: 0o755,
            nlink: 2,
            uid: 0,
            gid: 0,
            rdev: 0,
            flags: 0,
        };
        OcipkgFS {
            attr,
            inode_count: ROOT_INODE + 1,
            containers: Vec::new(),
            paths: BTreeMap::new(),
            attrs: HashMap::new(),
        }
    }

    /// Load OCI archive
    pub fn append_archive(&mut self, _path: impl AsRef<Path>) {
        // TODO moc
        let name = ImageName::default();
        self.containers.push(Container {
            base_ino: 0,
            name,
            attrs: HashMap::new(),
            paths: Vec::new(),
        });
        self.attr.nlink += 1;
    }

    fn new_file_attr(&mut self, size: u64) -> FileAttr {
        let ino = self.inode_count;
        self.inode_count += 1;
        FileAttr {
            ino,
            size,
            blocks: size / 512,
            atime: UNIX_EPOCH,
            mtime: UNIX_EPOCH,
            ctime: UNIX_EPOCH,
            crtime: UNIX_EPOCH,
            kind: FileType::RegularFile,
            perm: 0o644,
            // number of hard link for file is usually 1
            // unless explicit hard link exists.
            nlink: 1,
            uid: 0,
            gid: 0,
            rdev: 0,
            flags: 0,
        }
    }

    fn new_dir_attr(&mut self, num_subdirs: u32) -> FileAttr {
        let ino = self.inode_count;
        self.inode_count += 1;
        FileAttr {
            ino,
            size: 0,
            blocks: 0,
            atime: UNIX_EPOCH,
            mtime: UNIX_EPOCH,
            ctime: UNIX_EPOCH,
            crtime: UNIX_EPOCH,
            kind: FileType::Directory,
            perm: 0o755,
            nlink: num_subdirs + 2, /* from parent + `.` */
            uid: 0,
            gid: 0,
            rdev: 0,
            flags: 0,
        }
    }

    fn look_up(&self, parent: u64, name: &OsStr) -> Result<&FileAttr> {
        bail!("Not implemented yet, parent={parent}, name={name:?}");
    }

    /// Internal impl for [Filesystem::getattr]
    fn get_attr(&self, ino: u64) -> Result<&FileAttr> {
        if ino == ROOT_INODE {
            return Ok(&self.attr);
        }
        for c in &self.containers {
            if let Some(attr) = c.get_attr(ino) {
                return Ok(attr);
            }
        }
        bail!("Unknown inode");
    }

    fn read_dir(&self, ino: u64) -> Result<Vec<(u64, FileType, &str)>> {
        bail!("Unknown inode: {ino}");
    }
}

/// This implementations will pass arguments from filesystem call
/// to corresponding methods in `OcipkgFS`,
/// and convert runtime errors into `Reply` style.
impl Filesystem for OcipkgFS {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        match self.look_up(parent, name) {
            Ok(attr) => reply.entry(&TTL, attr, 0 /* generation */),
            Err(e) => {
                log::error!(target: "ocipkgfs::lookup", "{e}");
                reply.error(ENOENT);
            }
        }
    }

    /// See `OcipkgFS::get_attr`
    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        match self.get_attr(ino) {
            Ok(attr) => reply.attr(&TTL, &attr),
            Err(e) => {
                log::error!(target: "ocipkgfs::getattr", "{}", e);
                reply.error(ENOENT);
            }
        }
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        _size: u32,
        reply: ReplyData,
    ) {
        log::error!(target: "ocipkgfs::read", "ino = {ino}, offset = {offset}");
        reply.error(ENOENT);
    }

    /// See `OcipkgFS::read_dir`
    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        match self.read_dir(ino) {
            Ok(entries) => {
                for (i, (ino, ty, name)) in entries.into_iter().enumerate().skip(offset as usize) {
                    let offset = (i + 1) as i64; // i + 1 means the index of the next entry
                    reply.add(ino, offset, ty, name);
                }
                reply.ok();
            }
            Err(e) => {
                log::error!(target: "ocipkgfs::readdir", "{}", e);
                reply.error(ENOENT);
            }
        }
    }
}

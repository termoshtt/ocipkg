use flate2::read::GzDecoder;
use fuse::{
    FileAttr, FileType, Filesystem, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry, Request,
};
use libc::ENOENT;
use oci_spec::image::MediaType;
use ocipkg::{error::*, *};
use std::{
    ffi::OsStr,
    fs,
    path::*,
    sync::atomic::{AtomicU64, Ordering},
};
use time::Timespec;

/// Time to live (TTL) of filesystem cache
const TTL: Timespec = Timespec { sec: 1, nsec: 0 };
const UNIX_EPOCH: Timespec = Timespec { sec: 0, nsec: 0 }; // 1970-01-01 00:00:00

#[derive(Debug, Clone)]
enum Entry {
    Dir(DirEntry),
    File(FileEntry),
}

#[derive(Debug, Clone)]
struct FileEntry {
    attr: FileAttr,
}

#[derive(Debug, Clone)]
struct DirEntry {
    attr: FileAttr,
    contents: Vec<Entry>,
}

/// Cached metadata of a container or a manifest
#[derive(Debug, Clone)]
struct Container {
    name: ocipkg::ImageName,
    root: DirEntry,
}

/// Directory structure and their inodes should be like following diagram:
///
/// ```text
/// ocipkg root [inode=1]
///  │
///  ├─ some.registry_container_name__tag1/ [inode=2]
///  │  └─ dir1/       [inode=3]
///  │     └─ file1    [inode=4]
///  │
///  └─ another.registry_container_name__tag2/ [inode=5]
///     └─ dir2/       [inode=6]
///        └─ file2    [inode=7]
/// ```
///
/// - The contents in a container will be placed under the directory
///   corresponding to the container.
///
/// - Names of directories corresponding to container must be escaped.
///   e.g. `some.registry/container/name:tag1` will be escaped to
///   `some.registry_container_name__tag1`. See [ocipkg::ImageName::escaped] for detail.
///
/// - The inodes of contents are larger than that of the directory,
///   and smaller than the directory corresponding to the next container.
///   e.g. the inode of `dir1` (3) and `file1` (4) in above example are smaller
///   than the inode of `another.registry/container/name:tag2` (5).
///
#[derive(Debug, Clone)]
pub struct OcipkgFS {
    inode_count: u64,
    containers: Vec<Container>,
}

impl OcipkgFS {
    pub fn new() -> Self {
        OcipkgFS {
            inode_count: 0,
            containers: Vec::new(),
        }
    }

    /// Load OCI archive
    pub fn append_archive(&mut self, _path: impl AsRef<Path>) {
        // TODO moc
        const HELLO_TXT: &str = "Hello FUSE!\n";
        let name = ImageName::default();
        let file_attr = self.new_file_attr(HELLO_TXT.len() as u64);
        let dir_attr = self.new_dir_attr(0);
        let root = DirEntry {
            attr: dir_attr,
            contents: vec![Entry::File(FileEntry { attr: file_attr })],
        };
        self.containers.push(Container { name, root })
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

    fn debug_setup(&mut self) {
        const HELLO_TXT: &str = "Hello FUSE!\n";
        let name = ImageName::default();
        let hello_txt_entry = self.new_file_attr(HELLO_TXT.len() as u64);
    }
}

impl Filesystem for OcipkgFS {}

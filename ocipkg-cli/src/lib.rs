use anyhow::{bail, ensure, Result};
use fuse::{
    FileAttr, FileType, Filesystem, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry, Request,
};
use libc::ENOENT;
use ocipkg::*;
use std::{ffi::OsStr, path::*};
use time::Timespec;

/// Time to live (TTL) of filesystem cache
const TTL: Timespec = Timespec { sec: 1, nsec: 0 };
/// The UNIX epoch, `1970-01-01 00:00:00`
const UNIX_EPOCH: Timespec = Timespec { sec: 0, nsec: 0 };
/// Inode of filesystem root
const ROOT_INODE: u64 = 1;

#[derive(Debug, Clone)]
enum Entry {
    Dir(DirEntry),
    File(FileEntry),
}

impl Entry {
    fn get_attr(&self, ino: u64) -> Result<&FileAttr> {
        match self {
            Entry::Dir(dir) => dir.get_attr(ino),
            Entry::File(f) => {
                if ino == f.attr.ino {
                    Ok(&f.attr)
                } else {
                    bail!("Not found")
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
struct FileEntry {
    attr: FileAttr,
}

#[derive(Debug, Clone)]
struct DirEntry {
    attr: FileAttr,
    contents: Vec<Entry>,

    /// Number of sub-directories under this directory.
    ///
    /// This must be `0` if this directory only has files.
    num_subdirs: u64,
}

impl DirEntry {
    fn get_attr(&self, ino: u64) -> Result<&FileAttr> {
        if ino == self.attr.ino {
            return Ok(&self.attr);
        }
        // FIXME this should not be linear search
        for entry in &self.contents {
            if let Ok(attr) = entry.get_attr(ino) {
                return Ok(attr);
            }
        }
        bail!("Not found")
    }
}

/// Cached metadata of a container
#[derive(Debug, Clone)]
struct Container {
    /// Image name
    name: ocipkg::ImageName,
    /// Escaped image name
    escaped_name: String,
    /// Root of the filesystem tree
    root: DirEntry,
}

impl Container {
    fn get_attr(&self, ino: u64) -> Result<&FileAttr> {
        let mine = self.root.attr.ino;
        ensure!(ino < mine);
        if ino == mine {
            return Ok(&self.root.attr);
        }
        self.root.get_attr(ino)
    }
}

/// Directory structure and their inodes should be like following diagram:
///
/// ```text
/// ocipkg root [inode=1(ROOT_INODE)]
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
    attr: FileAttr,
    inode_count: u64,
    containers: Vec<Container>,
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
            num_subdirs: 0,
        };
        self.containers.push(Container {
            name,
            escaped_name: "test_container".to_string(),
            root,
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

    /// Get reference to the container which contains a file
    /// corresponding to the given inode.
    fn get_container_from_inode(&self, ino: u64) -> Result<&Container> {
        let mut index = self.containers.len();
        for (n, c) in self.containers.iter().enumerate() {
            if ino > c.root.attr.ino {
                index = n;
            }
        }
        if index == self.containers.len() {
            bail!("No container found for given inode {}", ino);
        } else {
            Ok(&self.containers[index])
        }
    }

    fn look_up(&self, parent: u64, name: &OsStr) -> Result<&FileAttr> {
        if parent == ROOT_INODE {
            for c in &self.containers {
                if let Some(name) = name.to_str() {
                    if name == c.escaped_name {
                        return Ok(&c.root.attr);
                    }
                }
            }
        }
        bail!("Not implemented yet, parent={parent}, name={name:?}");
    }

    /// Internal impl for [Filesystem::getattr]
    fn get_attr(&self, ino: u64) -> Result<&FileAttr> {
        if ino == ROOT_INODE {
            return Ok(&self.attr);
        }
        if ino > self.inode_count {
            bail!("Unknown inode");
        }
        self.get_container_from_inode(ino)?.get_attr(ino)
    }

    fn read_dir(&self, ino: u64) -> Result<Vec<(u64, FileType, &str)>> {
        if ino == ROOT_INODE {
            let mut entries = vec![
                (1, FileType::Directory, "."),
                (1, FileType::Directory, ".."),
            ];
            for c in &self.containers {
                entries.push((c.root.attr.ino, FileType::Directory, &c.escaped_name));
            }
            return Ok(entries);
        }
        let c = self.get_container_from_inode(ino)?;
        if c.root.attr.ino == ino {
            // TODO empty dir
            let entries = vec![
                (1, FileType::Directory, ".."),
                (ino, FileType::Directory, "."),
            ];
            return Ok(entries);
        }
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

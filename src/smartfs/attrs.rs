use fuser::{FileAttr, FileType};
use std::os::unix::fs::MetadataExt;
use std::{
    fs,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub const TTL: Duration = Duration::from_secs(1);

pub fn meta_to_attr(meta: &fs::Metadata, ino: u64) -> FileAttr {
    let kind = if meta.is_dir() {
        FileType::Directory
    } else if meta.file_type().is_symlink() {
        FileType::Symlink
    } else {
        FileType::RegularFile
    };

    FileAttr {
        ino,
        size: meta.size(),
        blocks: meta.blocks(),
        atime: UNIX_EPOCH + Duration::new(meta.atime() as u64, meta.atime_nsec() as u32),
        mtime: UNIX_EPOCH + Duration::new(meta.mtime() as u64, meta.mtime_nsec() as u32),
        ctime: UNIX_EPOCH + Duration::new(meta.ctime() as u64, meta.ctime_nsec() as u32),
        crtime: UNIX_EPOCH,
        kind,
        perm: (meta.mode() & 0o7777) as u16,
        nlink: meta.nlink() as u32,
        uid: meta.uid(),
        gid: meta.gid(),
        rdev: meta.rdev() as u32,
        blksize: meta.blksize() as u32,
        flags: 0,
    }
}

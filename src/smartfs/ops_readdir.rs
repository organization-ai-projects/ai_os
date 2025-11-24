use crate::smartfs::SmartFs;
use fuser::{ReplyDirectory, Request};
use std::ffi::OsStr;
use std::os::unix::fs::MetadataExt;

fn io_err_to_errno(e: &std::io::Error) -> i32 {
    use std::io::ErrorKind::*;
    match e.kind() {
        NotFound => libc::ENOENT,
        PermissionDenied => libc::EACCES,
        AlreadyExists => libc::EEXIST,
        InvalidInput => libc::EINVAL,
        DirectoryNotEmpty => libc::ENOTEMPTY,
        _ => libc::EIO,
    }
}

pub fn readdir(
    fs: &mut SmartFs,
    _req: &Request<'_>,
    ino: u64,
    _fh: u64,
    offset: i64,
    mut reply: ReplyDirectory,
) {
    if offset != 0 {
        reply.ok();
        return;
    }

    let Some(path) = fs.path_for_ino(ino) else {
        reply.error(libc::ENOENT);
        return;
    };

    match std::fs::read_dir(&path) {
        Ok(entries) => {
            let mut offset = 1;
            let _ = reply.add(ino, offset, fuser::FileType::Directory, ".");
            offset += 1;
            let _ = reply.add(ino, offset, fuser::FileType::Directory, "..");
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    let file_type = if meta.is_dir() {
                        fuser::FileType::Directory
                    } else if meta.is_file() {
                        fuser::FileType::RegularFile
                    } else {
                        fuser::FileType::Symlink
                    };
                    offset += 1;
                    let _ = reply.add(meta.ino(), offset, file_type, entry.file_name());
                }
            }
            reply.ok();
        }
        Err(e) => reply.error(io_err_to_errno(&e)),
    }
}

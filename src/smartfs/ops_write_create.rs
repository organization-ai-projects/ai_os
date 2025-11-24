use crate::smartfs::{attrs, SmartFs};
use fuser::{ReplyCreate, ReplyEntry, ReplyWrite, Request};
use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::{self, Seek, Write};
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::OpenOptionsExt;

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

pub fn write(
    fs: &mut SmartFs,
    _req: &Request<'_>,
    ino: u64,
    _fh: u64,
    offset: i64,
    data: &[u8],
    _wf: u32,
    _flags: i32,
    _lock: Option<u64>,
    reply: ReplyWrite,
) {
    let Some(path) = fs.path_for_ino(ino) else {
        reply.error(libc::ENOENT);
        return;
    };

    match OpenOptions::new().write(true).open(&path) {
        Ok(mut file) => {
            if file.seek(io::SeekFrom::Start(offset as u64)).is_ok() {
                match file.write(data) {
                    Ok(bytes_written) => reply.written(bytes_written as u32),
                    Err(e) => reply.error(io_err_to_errno(&e)),
                }
            } else {
                reply.error(libc::EIO);
            }
        }
        Err(e) => reply.error(io_err_to_errno(&e)),
    }
}

pub fn create(
    fs: &mut SmartFs,
    _req: &Request<'_>,
    parent: u64,
    name: &OsStr,
    mode: i32, // Correction du type pour correspondre à `custom_flags`
    _flags: i32,
    reply: ReplyCreate,
) {
    let Some(parent_path) = fs.path_for_ino(parent) else {
        reply.error(libc::ENOENT);
        return;
    };

    let full_path = parent_path.join(name);
    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .custom_flags(mode) // Correction du type
        .open(&full_path)
    {
        Ok(_) => {
            if let Ok(meta) = std::fs::metadata(&full_path) {
                let ino = meta.ino();
                let attr = attrs::meta_to_attr(&meta, ino); // Passe une référence à `meta`
                reply.created(&attrs::TTL, &attr, 0, 0, 0);
            } else {
                reply.error(libc::EIO);
            }
        }
        Err(e) => reply.error(io_err_to_errno(&e)),
    }
}

pub fn mkdir(
    fs: &mut SmartFs,
    _req: &Request<'_>,
    parent: u64,
    name: &OsStr,
    mode: u32,
    reply: ReplyEntry,
) {
    let Some(parent_path) = fs.path_for_ino(parent) else {
        reply.error(libc::ENOENT);
        return;
    };

    let full_path = parent_path.join(name);
    match fs::create_dir(&full_path) {
        Ok(_) => {
            if let Ok(meta) = fs::metadata(&full_path) {
                let attr = attrs::meta_to_attr(&meta, meta.ino());
                reply.entry(&attrs::TTL, &attr, 0);
            } else {
                reply.error(libc::EIO);
            }
        }
        Err(e) => reply.error(io_err_to_errno(&e)),
    }
}

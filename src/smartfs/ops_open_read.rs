use crate::smartfs::SmartFs;
use fuser::{ReplyData, ReplyOpen, Request};
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, Read, Seek};

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

pub fn open(fs: &mut SmartFs, _req: &Request<'_>, ino: u64, _flags: i32, reply: ReplyOpen) {
    let Some(path) = fs.path_for_ino(ino) else {
        reply.error(libc::ENOENT);
        return;
    };

    if path.exists() {
        reply.opened(0, 0);
    } else {
        reply.error(libc::ENOENT);
    }
}

pub fn read(
    fs: &mut SmartFs,
    _req: &Request<'_>,
    ino: u64,
    _fh: u64,
    offset: i64,
    size: u32,
    reply: ReplyData,
) {
    let Some(path) = fs.path_for_ino(ino) else {
        reply.error(libc::ENOENT);
        return;
    };

    match File::open(&path) {
        Ok(mut file) => {
            let mut buffer = vec![0; size as usize];
            if file.seek(io::SeekFrom::Start(offset as u64)).is_ok() {
                match file.read(&mut buffer) {
                    Ok(bytes_read) => reply.data(&buffer[..bytes_read]),
                    Err(e) => reply.error(io_err_to_errno(&e)),
                }
            } else {
                reply.error(libc::EIO);
            }
        }
        Err(e) => reply.error(io_err_to_errno(&e)),
    }
}

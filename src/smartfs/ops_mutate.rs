use super::SmartFs;
use fuser::{ReplyEmpty, Request};
use std::{ffi::OsStr, fs, os::unix::fs::MetadataExt, path::PathBuf};

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

pub fn unlink(fs: &mut SmartFs, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEmpty) {
    let Some(parent_path) = fs.path_for_ino(parent) else {
        reply.error(libc::ENOENT);
        return;
    };

    let full_path: PathBuf = parent_path.join(name);

    match fs::remove_file(&full_path) {
        Ok(_) => {
            // optionnel V0: nettoyer un ino si tu le stockes par path
            reply.ok()
        }
        Err(e) => reply.error(io_err_to_errno(&e)),
    }
}

pub fn rmdir(fs: &mut SmartFs, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEmpty) {
    let Some(parent_path) = fs.path_for_ino(parent) else {
        reply.error(libc::ENOENT);
        return;
    };

    let full_path: PathBuf = parent_path.join(name);

    match fs::remove_dir(&full_path) {
        Ok(_) => reply.ok(),
        Err(e) => reply.error(io_err_to_errno(&e)),
    }
}

pub fn rename(
    fs: &mut SmartFs,
    _req: &Request<'_>,
    parent: u64,
    name: &OsStr,
    newparent: u64,
    newname: &OsStr,
    reply: ReplyEmpty,
) {
    let Some(old_parent_path) = fs.path_for_ino(parent) else {
        reply.error(libc::ENOENT);
        return;
    };
    let Some(new_parent_path) = fs.path_for_ino(newparent) else {
        reply.error(libc::ENOENT);
        return;
    };

    let old_path: PathBuf = old_parent_path.join(name);
    let new_path: PathBuf = new_parent_path.join(newname);

    match fs::rename(&old_path, &new_path) {
        Ok(_) => {
            // Update cache best-effort (évite les inodes stale)
            if let Ok(meta) = fs::symlink_metadata(&new_path) {
                fs.cache_ino_path(meta.ino(), new_path);
            }
            reply.ok()
        }
        Err(e) => reply.error(io_err_to_errno(&e)),
    }
}

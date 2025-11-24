use crate::smartfs::{attrs, SmartFs};
use fuser::{ReplyAttr, ReplyEntry, Request};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::sync::RwLock;

lazy_static! {
    static ref METADATA_CACHE: RwLock<HashMap<String, std::fs::Metadata>> =
        RwLock::new(HashMap::new());
}

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

pub fn lookup(fs: &mut SmartFs, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEntry) {
    let Some(parent_path) = fs.path_for_ino(parent) else {
        reply.error(libc::ENOENT);
        return;
    };

    let full_path = parent_path.join(name);
    let full_path_str = full_path.to_string_lossy().to_string();

    let meta = {
        let cache = METADATA_CACHE.read().unwrap();
        cache.get(&full_path_str).cloned()
    };

    let meta = meta.or_else(|| {
        let new_meta = std::fs::metadata(&full_path).ok();
        if let Some(ref m) = new_meta {
            let mut cache = METADATA_CACHE.write().unwrap();
            cache.insert(full_path_str.clone(), m.clone());
        }
        new_meta
    });

    match meta {
        Some(meta) => {
            let attr = attrs::meta_to_attr(&meta, parent);
            reply.entry(&attrs::TTL, &attr, 0);
        }
        None => reply.error(libc::ENOENT),
    }
}

pub fn getattr(fs: &mut SmartFs, _req: &Request<'_>, ino: u64, reply: ReplyAttr) {
    let Some(path) = fs.path_for_ino(ino) else {
        reply.error(libc::ENOENT);
        return;
    };

    let path_str = path.to_string_lossy().to_string();

    let meta = {
        let cache = METADATA_CACHE.read().unwrap();
        cache.get(&path_str).cloned()
    };

    let meta = meta.or_else(|| {
        let new_meta = std::fs::metadata(&path).ok();
        if let Some(ref m) = new_meta {
            let mut cache = METADATA_CACHE.write().unwrap();
            cache.insert(path_str.clone(), m.clone());
        }
        new_meta
    });

    match meta {
        Some(meta) => {
            let attr = attrs::meta_to_attr(&meta, ino);
            reply.attr(&attrs::TTL, &attr);
        }
        None => reply.error(libc::ENOENT),
    }
}

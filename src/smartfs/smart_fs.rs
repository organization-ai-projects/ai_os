use anyhow::{Context, Result};
use fuser::{
    Filesystem, ReplyAttr, ReplyCreate, ReplyData, ReplyDirectory, ReplyEmpty, ReplyEntry,
    ReplyOpen, ReplyWrite, Request,
};
use std::os::unix::fs::MetadataExt;
use std::{
    fs,
    path::{Path, PathBuf},
};

use super::{attrs, inode_cache::InodeCache};

pub struct SmartFs {
    pub root: PathBuf,
    pub cache: InodeCache,
}

impl SmartFs {
    pub fn new(root: PathBuf) -> Result<Self> {
        let root = root.canonicalize().context("real_dir does not exist")?;
        let cache = InodeCache::new();
        let meta = fs::metadata(&root)?;
        cache.insert(meta.ino(), root.clone());

        Ok(Self { root, cache })
    }

    pub fn path_for_ino(&self, ino: u64) -> Option<PathBuf> {
        self.cache.get(ino)
    }

    pub fn cache_ino_path(&self, ino: u64, path: PathBuf) {
        self.cache.insert(ino, path)
    }

    pub fn full_path(&self, p: &Path) -> PathBuf {
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            self.root.join(p)
        }
    }
}

impl Filesystem for SmartFs {
    fn lookup(
        &mut self,
        req: &Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        reply: ReplyEntry,
    ) {
        super::ops_lookup::lookup(self, req, parent, name, reply)
    }

    fn getattr(&mut self, req: &Request<'_>, ino: u64, reply: ReplyAttr) {
        super::ops_lookup::getattr(self, req, ino, reply)
    }

    fn readdir(
        &mut self,
        req: &Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        reply: ReplyDirectory,
    ) {
        super::ops_readdir::readdir(self, req, ino, fh, offset, reply)
    }

    fn open(&mut self, req: &Request<'_>, ino: u64, flags: i32, reply: ReplyOpen) {
        super::ops_open_read::open(self, req, ino, flags, reply)
    }

    fn read(
        &mut self,
        req: &Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        size: u32,
        flags: i32,
        lock: Option<u64>,
        reply: ReplyData,
    ) {
        super::ops_open_read::read(self, req, ino, fh, offset, size, reply);
    }

    fn write(
        &mut self,
        req: &Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        data: &[u8],
        wf: u32,
        flags: i32,
        lock: Option<u64>,
        reply: ReplyWrite,
    ) {
        super::ops_write_create::write(self, req, ino, fh, offset, data, wf, flags, lock, reply)
    }

    fn create(
        &mut self,
        req: &Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        mode: u32,
        umask: u32,
        flags: i32,
        reply: ReplyCreate,
    ) {
        super::ops_write_create::create(self, req, parent, name, mode as i32, flags, reply);
    }

    fn mkdir(
        &mut self,
        req: &Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        mode: u32,
        umask: u32,
        reply: ReplyEntry,
    ) {
        super::ops_write_create::mkdir(self, req, parent, name, mode, reply);
    }

    fn unlink(
        &mut self,
        req: &Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        reply: ReplyEmpty,
    ) {
        super::ops_mutate::unlink(self, req, parent, name, reply)
    }

    fn rmdir(&mut self, req: &Request<'_>, parent: u64, name: &std::ffi::OsStr, reply: ReplyEmpty) {
        super::ops_mutate::rmdir(self, req, parent, name, reply)
    }

    fn rename(
        &mut self,
        req: &Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        newparent: u64,
        newname: &std::ffi::OsStr,
        flags: u32,
        reply: ReplyEmpty,
    ) {
        super::ops_mutate::rename(self, req, parent, name, newparent, newname, reply);
    }
}

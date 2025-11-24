use dashmap::DashMap;
use std::path::PathBuf;

#[derive(Clone, Default)]
pub struct InodeCache {
    map: DashMap<u64, PathBuf>,
}

impl InodeCache {
    pub fn new() -> Self {
        Self {
            map: DashMap::new(),
        }
    }

    pub fn get(&self, ino: u64) -> Option<PathBuf> {
        self.map.get(&ino).map(|entry| entry.clone())
    }

    pub fn insert(&self, ino: u64, path: PathBuf) {
        self.map.insert(ino, path);
    }
}

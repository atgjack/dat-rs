use std::io::Result;
use lru_cache::LruCache;

use core::storage::{Storage, FileType};
use common::merkle::Node;

pub struct CachedStorage<T: Storage> {
    cache:          LruCache<u64, Node>,
    storage:        T
}

impl<T: Storage> CachedStorage<T> {
    pub fn new(storage: T) -> CachedStorage<T> {
        CachedStorage{
            cache:          LruCache::new(65536),
            storage:        storage,
        }
    }
}

impl<T: Storage> Storage for CachedStorage<T> {
    fn read_archive(&mut self, file_type: FileType, offset: u64, buf: &mut [u8]) -> Result<usize> {
        self.storage.read_archive(file_type, offset, buf)
    }

    fn write_archive(&mut self, file_type: FileType, offset: u64, buf: &[u8]) -> Result<()> {
        self.storage.write_archive(file_type, offset, buf)
    }

    fn get_node(&mut self, index: u64) -> Result<Option<Node>> {
        if let Some(node) = self.cache.get_mut(&index) {
            return Ok(Some(node.clone()));
        }

        match try!(self.storage.get_node(index)) {
            None        => Ok(None),
            Some(node)  => {
                self.cache.insert(index, node.clone());
                Ok(Some(node))
            },
        }
    }
} 
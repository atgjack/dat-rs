extern crate dat;

use std::fs::remove_dir_all;
use std::path::Path;

use dat::core::Hypercore;
use dat::core::storage::{FileStorage, CachedStorage, MemoryStorage};

const DIR_PATH: &str = "/home/vader/test";

fn cleanup() {
    let path = Path::new(DIR_PATH).join(".dat");
    
    let _ = remove_dir_all(path);
    println!(" ");
    println!("Reset test directory.");
    println!(" ");
}

#[test]
fn test_file_storage() {
    cleanup();

    let path = Path::new(DIR_PATH);
    let storage = FileStorage::new(path).unwrap();
    let mut feed = Hypercore::new(storage).unwrap();
    let data = vec![0u8; 1024 * 64];

    for i in 0..64 {
        feed.append(data.clone()).unwrap();
        assert_eq!(feed.get(i).unwrap().unwrap(), data.clone());
    }
}

#[test]
fn test_memory_storage() {
    let storage = MemoryStorage::new();
    let mut feed = Hypercore::new(storage).unwrap();
    let data = vec![0u8; 1024 * 64];

    for i in 0..64  {
        feed.append(data.clone()).unwrap();
        assert_eq!(feed.get(i).unwrap().unwrap(), data.clone());
    }
}

#[test]
fn test_cached_storage() {
    let storage = MemoryStorage::new();
    let cached = CachedStorage::new(storage);
    let mut feed = Hypercore::new(cached).unwrap();
    let data = vec![0u8; 1024 * 64];

    for i in 0..64  {
        feed.append(data.clone()).unwrap();
        for _ in 0..64 {
            assert_eq!(feed.get(i).unwrap().unwrap(), data.clone());
        }
    }
}


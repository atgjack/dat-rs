extern crate dat;

use std::fs::remove_dir_all;
use std::path::Path;

use dat::core::feed::Feed;

const DIR_PATH: &str = "/home/vader/test";

fn cleanup() {
    let path = Path::new(DIR_PATH).join(".dat");
    
    remove_dir_all(path).unwrap();
    println!(" ");
    println!("Reset test directory.");
    println!(" ");
}

#[test]
fn test_can_open_dir() {
    cleanup();

    let path = Path::new(DIR_PATH);
    let mut feed = Feed::new(path).unwrap();
    let data = vec![0u8; 1024 * 64];

    for i in 0..(1024 * 1) {
        feed.append(data.clone()).unwrap();
        assert_eq!(feed.get(i).unwrap().unwrap(), data.clone());
    }
}

extern crate dat;
extern crate blake2;

use blake2::Blake2b;

use dat::common::merkle::Tree;

#[test]
fn test_one_root_on_power_of_two() {
    let mut tree = Tree::new();
    tree.insert::<Blake2b>(b"test".to_vec());
    tree.insert::<Blake2b>(b"test".to_vec());
    tree.insert::<Blake2b>(b"test".to_vec());
    tree.insert::<Blake2b>(b"test".to_vec());

    assert_eq!(tree.roots.len(), 1);
}

#[test]
fn test_multiple_roots_if_not_power_of_two() {
    let mut tree = Tree::new();
    tree.insert::<Blake2b>(b"test".to_vec());
    tree.insert::<Blake2b>(b"test".to_vec());
    tree.insert::<Blake2b>(b"test".to_vec());
    tree.insert::<Blake2b>(b"test".to_vec());
    tree.insert::<Blake2b>(b"test".to_vec());

    assert!(tree.roots.len() > 1);
}

#[test]
fn test_can_handle_large_trees() {
    let mut tree = Tree::new();
    const NUM: usize = 1024;

    for _ in 0..NUM {
        tree.insert::<Blake2b>(b"test".to_vec());
    }

    assert!(tree.roots.len() == 1);

    tree.insert::<Blake2b>(b"test".to_vec());
    assert!(tree.roots.len() > 1);    
}
extern crate dat;
extern crate rand;

use rand::Rng;
use std::usize::MAX;
use dat::common::varint;

fn gen_ints(num: usize) -> Vec<usize> {
    let mut rng = rand::thread_rng();
    let mut result: Vec<usize> = Vec::with_capacity(num);
    for _ in 0..num {
        result.push(rng.gen_range(0, MAX))
    }
    result
}

#[test]
fn test_single_byte() {
    let val: usize = 300;
    let vec: Vec<u8> = vec![172,2];
    let encoded = varint::encode(val);
    let decoded = varint::decode(&encoded);
    assert_eq!(varint::length(val), vec.len());
    assert_eq!(encoded, vec);
    assert_eq!(val, decoded);
}

#[test]
fn test_fuzz() {
    for num in gen_ints(100) {
        let encoded = varint::encode(num);
        let decoded = varint::decode(&encoded);
        assert_eq!(encoded.len(), varint::length(num));
        assert_eq!(num, decoded);
    }
}
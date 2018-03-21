use std::cell::RefCell;
use std::rc::Rc;
use std::ops::{Range};

use common::flat;
use common::pager::Pager;
use common::sparse::SparseBitfield;

pub struct Bitfield {
    pager:      Rc<RefCell<Pager>>,
    data:       SparseBitfield,
    index:      SparseBitfield,
    tree:       SparseBitfield,
}

impl Bitfield {
    fn with_pager(pager: Rc<RefCell<Pager>>) -> Bitfield {
        Bitfield {
            pager:      Rc::clone(&pager),
            data:       SparseBitfield::with_pager(Rc::clone(&pager), 0, 1024),
            tree:       SparseBitfield::with_pager(Rc::clone(&pager), 1024, 2048),
            index:      SparseBitfield::with_pager(Rc::clone(&pager), 1024 + 2048, 256),
        }
    }

    pub fn new() -> Bitfield {
        let pager = Rc::new(RefCell::new(Pager::new()));
        Bitfield::with_pager(pager)
    }

    pub fn from_vec(vec: Vec<u8>) -> Bitfield {
        let pager = Rc::new(RefCell::new(Pager::from_vec(vec)));
        Bitfield::with_pager(pager)
    }

    pub fn get(&self, index: u64) -> bool {
        self.data.get(index)
    }

    pub fn set(&mut self, index: u64, value: bool) -> bool {
        self.set_data(index, value) && 
        self.set_tree(index, value) &&
        self.set_index(index)
    }

    fn set_data(&mut self, index: u64, value: bool) -> bool {
        self.data.set(index, value)
    }

    fn set_tree(&mut self, index: u64, value: bool) -> bool {
        let mut current = index * 2;
        if !self.tree.set(current, value) { return false; }
        while self.tree.get(flat::sibling(current)) {
            current = flat::parent(current);
            if !self.tree.set(current, true) { break; }
        }
        true
    }

    fn set_index(&mut self, index: u64) -> bool {
        let value = self.data.get_byte(index);
        let o = index as usize & 3;
        let start = index * 2;
        let tup = match value {
            255 => 0b11000000,
            0   => 0b00000000,
            _   => 0b01000000
        };
        let mask: u8 = !(3 << (6 - (o * 2)));
        let mut byte = (self.index.get_byte(start) & mask) | tup << (2 * 0);
        let max_len = self.index.pages() * 256;

        let mut current = start;

        while current < max_len && self.index.set_byte(index, value) {
            let sibling = self.index.get_byte(flat::sibling(current));
            if flat::is_left(current) {
                byte = (convert_to_index(byte) << 4) | convert_to_index(sibling);
            } else {
                byte = (convert_to_index(sibling) << 4) | convert_to_index(byte);
            }

            current = flat::parent(current);
        }

        current != start
    }

    pub fn total(&self, range: Range<u64>) -> u64 {
        let start = (range.start & 7) as u8;
        let end = (range.end & 7) as u8;
        let pos = range.start - start as u64 / 8;
        let last = range.end - end as u64 / 8;
        let left_mask = match start {
            1   => 255 - 0b10000000,
            2   => 255 - 0b11000000,
            3   => 255 - 0b11100000,
            4   => 255 - 0b11110000,
            5   => 255 - 0b11111000,
            6   => 255 - 0b11111100,
            7   => 255 - 0b11111110,
            _   => 255,
        };
        let right_mask = match end {
            1   => 0b10000000,
            2   => 0b11000000,
            3   => 0b11100000,
            4   => 0b11110000,
            5   => 0b11111000,
            6   => 0b11111100,
            7   => 0b11111110,
            _   => 0,
        };
        let byte = self.data.get_byte(pos);
        if pos == last {
            return (byte & left_mask & right_mask).count_ones() as u64;
        }
        let mut total = (byte & left_mask).count_ones() as u64;
        for i in (pos + 1)..last {
            total += self.data.get_byte(i).count_ones() as u64;
        }
        total += (self.data.get_byte(last) & right_mask).count_ones() as u64;
        total
    }

    pub fn blocks(&self) -> u64 {
        let mut top = 0;
        let mut next = 0;
        let max = self.tree.len();

        while flat::right_span(next) < max {
            next = flat::parent(next);
            if self.tree.get(next) {
                top = next;
            }
        }

        if !self.tree.get(top) {
            return 0;
        }
        

        match self.verfied_by(top) {
            Some(val) => val / 2,
            None      => 0,
        }
    }

    pub fn roots(&self) -> Vec<u64> {
        flat::full_roots(2 * self.blocks())
    }

    pub fn verfied_by(&self, index: u64) -> Option<u64> {
        if !self.tree.get(index) { return None; }
        let mut depth = flat::depth(index);
        let mut top = index;
        let mut parent = flat::parent_with_depth(index, depth);
        depth += 1;

        // Find current root.
        while self.tree.get(parent) && self.tree.get(flat::sibling(top)) {
            top = parent;
            parent = flat::parent_with_depth(top, depth);
            depth += 1;
        }

        // Extend right down.
        depth -= 1;
        while depth > 0 {
            parent = flat::index(depth, flat::offset_with_depth(top, depth) + 1);
            top = match flat::child_with_depth(parent, depth, true) {
                Some(child) => child,
                None => return None,   
            };
            depth -= 1;

            while !self.tree.get(top) && depth > 0 { 
                top = match flat::child_with_depth(top, depth, true) {
                    Some(child) => child,
                    None => return None,   
                };
                depth -= 1;
            }
        }

        match self.tree.get(top) {
            true    => Some(top + 2),
            false   => Some(top),
        }
    }

    pub fn digest(&self, index: u64) -> u64 {
        if self.tree.get(index) {
            return 1;
        }

        let mut digest = 0u64;
        let mut next = flat::sibling(index);
        let max = match next + 2 > self.tree.len() {
            true    => next + 2,
            false   => self.tree.len()
        };

        let mut bit = 2u64;
        let mut depth = flat::depth(index);
        let mut parent = flat::parent_with_depth(next, depth);
        depth += 1;

        while flat::right_span(next) < max || flat::left_span(parent) > 0 {
            if self.tree.get(next) {
                digest |= bit;
            }

            if self.tree.get(parent) {
                digest |= 2 * bit + 1;
                if digest + 1 == 4 * bit {
                    return 1;
                }
                return digest;
            }

            next = flat::sibling(parent);
            parent = flat::parent_with_depth(next, depth);
            depth += 1;
            bit *= 2;
        }

        digest
    }

    // pub fn proof(&self, index: u64, opts: ProofOpts) -> Option<(Vec<u64>, u64)> {
    //     if !self.get(index) { return None; }
        
    //     let mut nodes: Vec<u64> = Vec::new();
    //     let mut digest = opts.digest;
    //     let mut remote = opts.remote;
        
    //     if opts.hash { nodes.push(index); }
    //     if digest == 1 { return Some((nodes, 0)); }

    //     let mut roots: Vec<u64>;
    //     let mut sibling;
    //     let mut next = index;
        
    //     digest >>= 1;
    //     while digest > 0 {
    //         if digest == 1 && digest & 1 != 0 {
    //             if self.get(next) { remote.set(next); }
    //             if flat::sibling(next) < next { next = flat::sibling(next); }
    //             roots = flat::full_roots(flat::right_span(next) + 2);
    //             for &root in &roots {
    //                 if self.get(root) { remote.set(root); }
    //             }
    //             break;
    //         }

    //         sibling = flat::sibling(next);
    //         if digest & 1 > 0 && self.get(sibling) { remote.set(sibling); }
    //         next = flat::parent(next);
    //         digest >>= 1;
    //     }

    //     next = index;

    //     while !remote.get(next) {
    //         sibling = flat::sibling(next);
    //         if !self.get(sibling) {
    //             match self.verfied_by(next) {
    //                 None => return None,
    //                 Some(val) => {
    //                     roots = flat::full_roots(val);
    //                     for &root in &roots {
    //                         if root != next && !remote.get(root) { nodes.push(root); }
    //                     }
    //                     return Some((nodes, val));
    //                 }
    //             }
    //         } else {
    //             if !remote.get(sibling) { nodes.push(sibling); }
    //         }

    //         next = flat::parent(next);
    //     }

    //     Some((nodes, 0))
    // }

    pub fn to_vec(&self) -> Vec<u8> {
        let pager = self.pager.borrow();
        let page_size = pager.get_page_size();
        let mut result: Vec<u8> = vec![0u8; pager.len() * page_size as usize];
        for (&index, value) in pager.iter() {
            let start: usize = index * page_size;
            result[start..(start + page_size)].copy_from_slice(value.as_slice());
        }
        result
    }

    pub fn last_updated(&mut self) -> Option<(usize, Vec<u8>)> {
        let mut pager = self.pager.borrow_mut();
        match pager.last_updated() {
            None        => None,
            Some(num)   => Some((num * pager.get_page_size(), pager.get(num).unwrap().to_vec()))
        }
    }
}

// pub struct ProofOpts {
//     remote:     TreeBitfield,
//     digest:     u64,
//     hash:       bool,
// }

// impl ProofOpts {
//     pub fn new() -> ProofOpts {
//         ProofOpts {
//             remote:     TreeBitfield::new(),
//             digest:     0,
//             hash:       false,
//         }
//     }

//     pub fn set_remote(&mut self, remote: TreeBitfield) {
//         self.remote = remote;
//     }

//     pub fn set_digest(&mut self, digest: u64) {
//         self.digest = digest;
//     }

//     pub fn set_hash(&mut self, hash: bool) {
//         self.hash = hash;
//     }
// }

fn convert_to_index(value: u8) -> u8 {
    let left = match (value & (15 << 4)) >> 4 {
        15  => 0b00001100,
        0   => 0b00000000,
        _   => 0b00000100,
    };
    let right = match value & 15 {
        15  => 0b00000011,
        0   => 0b00000000,
        _   => 0b00000001,
    };
    left | right
}
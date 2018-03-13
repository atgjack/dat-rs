use std::collections::{HashMap};
use std::collections::hash_map::{Iter, Entry};
use std::cell::RefCell;
use std::rc::Rc;
use std::ops::{Range};
use indexmap::IndexSet;

use tree;

const PAGE_SIZE: usize = 3328;

pub struct Pager {
    map:        HashMap<usize, Vec<u8>>,
    updated:    IndexSet<usize>,
}

impl Pager {
    pub fn new() -> Pager {
        Pager {
            map:        HashMap::new(),
            updated:    IndexSet::new(),
        }
    }

    pub fn get(&self, index: usize) -> Option<&Vec<u8>> {
        self.map.get(&index)
    }

    pub fn set(&mut self, page_num: usize, byte_num: usize, value: u8) -> bool {
        match self.map.entry(page_num) {
            Entry::Vacant(page) => {
                if value == 0 { return false; }
                let mut vec = vec![0u8; PAGE_SIZE];
                vec[byte_num] = value;
                page.insert(vec);
            },
            Entry::Occupied(entry) => {
                let mut page = entry.into_mut();
                if page[byte_num] == value { return false; }
                page[byte_num] = value;
            }
        }
        self.updated.insert(page_num);
        true
    }

    pub fn insert(&mut self, index: usize, value: Vec<u8>) {
        self.map.insert(index, value);
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn iter(&self) -> Iter<usize, Vec<u8>> {
        self.map.iter()
    }

    pub fn last_updated(&mut self) -> Option<usize> {
        self.updated.pop()
    }
}

pub struct SparseBitfield {
    pager:      Rc<RefCell<Pager>>,
    offset:     usize,
    size:       usize,
}

impl SparseBitfield {
    pub fn new() -> SparseBitfield {
        SparseBitfield {
            pager:      Rc::new(RefCell::new(Pager::new())),
            offset:     0,
            size:       1024,
        }
    }

    pub fn with_pager(pager: Rc<RefCell<Pager>>, offset: usize, size: usize) -> SparseBitfield {
        SparseBitfield {
            pager:      pager,
            offset:     offset,
            size:       size,
        }
    }

    pub fn get(&self, index: u64) -> bool {
        self.get_byte(index) & self.get_offset(index) != 0
    }

    pub fn set(&mut self, index: u64, value: bool) -> bool {
        let mut byte = self.get_byte(index);
        match value {
            true    => byte |= self.get_offset(index),
            false   => byte &= !(self.get_offset(index)),
        }
        self.set_byte(index, byte)
    }

    pub fn get_byte(&self, index: u64) -> u8 {
        let page_num = self.get_page_num(index);
        let byte_num = self.get_byte_num(index);
        match self.pager.borrow().get(page_num) {
            None        => 0,
            Some(page)  => page[byte_num]
        }
    }

    pub fn set_byte(&mut self, index: u64, value: u8) -> bool {
        let page_num = self.get_page_num(index);
        let byte_num = self.get_byte_num(index);
        self.pager.borrow_mut().set(page_num, byte_num, value)
    }

    pub fn len(&self) -> u64 {
        self.pager.borrow().len() as u64 * PAGE_SIZE as u64 * 8
    }

    pub fn pages(&self) -> u64 {
        self.pager.borrow().len() as u64
    }

    fn get_offset(&self, index: u64) -> u8 {
        let offset = index & 7;
        1 << offset
    }

    fn get_page_num(&self, index: u64) -> usize {
        (index as usize) / (self.size * 8)
    }

    fn get_byte_num(&self, index: u64) -> usize {
        self.offset + ((index as usize / 8) & (self.size - 1))
    }
}

struct IndexBitfield {
    bitfield: SparseBitfield
}

impl IndexBitfield {
    pub fn with_bitfield(bitfield: SparseBitfield) -> IndexBitfield {
        IndexBitfield {
            bitfield: bitfield
        }
    }

    fn convert_to_index(&self, value: u8) -> u8 {
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

    pub fn set(&mut self, index: u64, value: u8) -> bool {
        let o = index as usize & 3;
        let start = index * 2;
        let tup = match value {
            255 => 0b11000000,
            0   => 0b00000000,
            _   => 0b01000000
        };
        let mask: u8 = !(3 << (6 - (o * 2)));
        let mut byte = (self.bitfield.get_byte(start) & mask) | tup << (2 * 0);
        let max_len = self.bitfield.pages() * 256;

        let mut current = start;

        while current < max_len && self.bitfield.set_byte(index, value) {
            let sibling = self.bitfield.get_byte(tree::sibling(current));
            if tree::is_left(current) {
                byte = (self.convert_to_index(byte) << 4) | self.convert_to_index(sibling);
            } else {
                byte = (self.convert_to_index(sibling) << 4) | self.convert_to_index(byte);
            }

            current = tree::parent(current);
        }

        current != start
    }
}

pub struct TreeIndex {
    bitfield: SparseBitfield
}

impl TreeIndex {
    pub fn new() -> TreeIndex {
        TreeIndex {
            bitfield:   SparseBitfield::new()
        }
    }

    pub fn with_bitfield(bitfield: SparseBitfield) -> TreeIndex {
        TreeIndex {
            bitfield: bitfield
        }
    }

    pub fn get(&self, index: u64) -> bool {
        self.bitfield.get(index)
    }

    pub fn set(&mut self, index: u64) -> bool {
        let mut current = index * 2;
        if !self.bitfield.set(current, true) { return false; }
        while self.bitfield.get(tree::sibling(current)) {
            current = tree::parent(current);
            if !self.bitfield.set(current, true) { break; }
        }
        true
    }

    pub fn verfied_by(&self, index: u64) -> Option<u64> {
        if !self.get(index) { return None; }
        let mut depth = tree::depth(index);
        let mut top = index;
        let mut parent = tree::parent_with_depth(index, depth);
        depth += 1;

        // Find current root.
        while self.get(parent) && self.get(tree::sibling(top)) {
            top = parent;
            parent = tree::parent_with_depth(top, depth);
            depth += 1;
        }

        // Extend right down.
        depth -= 1;
        while depth > 0 {
            parent = tree::index(depth, tree::offset_with_depth(top, depth) + 1);
            top = match tree::child_with_depth(parent, depth, true) {
                Some(child) => child,
                None => return None,   
            };
            depth -= 1;

            while !self.get(top) && depth > 0 { 
                top = match tree::child_with_depth(top, depth, true) {
                    Some(child) => child,
                    None => return None,   
                };
                depth -= 1;
            }
        }

        match self.get(top) {
            true    => Some(top + 2),
            false   => Some(top),
        }
    }

    pub fn blocks(&self) -> u64 {
        let mut top = 0;
        let mut next = 0;
        let max = self.bitfield.len();

        while tree::right_span(next) < max {
            next = tree::parent(next);
            if self.get(next) {
                top = next;
            }
        }

        if !self.get(top) {
            return 0;
        }
        

        match self.verfied_by(top) {
            Some(val) => val / 2,
            None      => 0,
        }
    }

    pub fn roots(&self) -> Vec<u64> {
        tree::full_roots(2 * self.blocks())
    }

    pub fn digest(&self, index: u64) -> u64 {
        if self.get(index) {
            return 1;
        }

        let mut digest = 0u64;
        let mut next = tree::sibling(index);
        let max = match next + 2 > self.bitfield.len() {
            true    => next + 2,
            false   => self.bitfield.len()
        };

        let mut bit = 2u64;
        let mut depth = tree::depth(index);
        let mut parent = tree::parent_with_depth(next, depth);
        depth += 1;

        while tree::right_span(next) < max || tree::left_span(parent) > 0 {
            if self.get(next) {
                digest |= bit;
            }

            if self.get(parent) {
                digest |= 2 * bit + 1;
                if digest + 1 == 4 * bit {
                    return 1;
                }
                return digest;
            }

            next = tree::sibling(parent);
            parent = tree::parent_with_depth(next, depth);
            depth += 1;
            bit *= 2;
        }

        digest
    }

    pub fn proof(&self, index: u64, opts: ProofOpts) -> Option<(Vec<u64>, u64)> {
        if !self.get(index) { return None; }
        
        let mut nodes: Vec<u64> = Vec::new();
        let mut digest = opts.digest;
        let mut remote = opts.remote;
        
        if opts.hash { nodes.push(index); }
        if digest == 1 { return Some((nodes, 0)); }

        let mut roots: Vec<u64>;
        let mut sibling;
        let mut next = index;
        
        digest >>= 1;
        while digest > 0 {
            if digest == 1 && digest & 1 != 0 {
                if self.get(next) { remote.set(next); }
                if tree::sibling(next) < next { next = tree::sibling(next); }
                roots = tree::full_roots(tree::right_span(next) + 2);
                for &root in &roots {
                    if self.get(root) { remote.set(root); }
                }
                break;
            }

            sibling = tree::sibling(next);
            if digest & 1 > 0 && self.get(sibling) { remote.set(sibling); }
            next = tree::parent(next);
            digest >>= 1;
        }

        next = index;

        while !remote.get(next) {
            sibling = tree::sibling(next);
            if !self.get(sibling) {
                match self.verfied_by(next) {
                    None => return None,
                    Some(val) => {
                        roots = tree::full_roots(val);
                        for &root in &roots {
                            if root != next && !remote.get(root) { nodes.push(root); }
                        }
                        return Some((nodes, val));
                    }
                }
            } else {
                if !remote.get(sibling) { nodes.push(sibling); }
            }

            next = tree::parent(next);
        }

        Some((nodes, 0))
    }
}

pub struct ProofOpts {
    remote:     TreeIndex,
    digest:     u64,
    hash:       bool,
}

impl ProofOpts {
    pub fn new() -> ProofOpts {
        ProofOpts {
            remote:     TreeIndex::new(),
            digest:     0,
            hash:       false,
        }
    }

    pub fn set_remote(&mut self, remote: TreeIndex) {
        self.remote = remote;
    }

    pub fn set_digest(&mut self, digest: u64) {
        self.digest = digest;
    }

    pub fn set_hash(&mut self, hash: bool) {
        self.hash = hash;
    }
}

pub struct Bitfield {
    pager:        Rc<RefCell<Pager>>,
    data:       SparseBitfield,
    index:      IndexBitfield,
}

impl Bitfield {
    fn with_pager(pager: Rc<RefCell<Pager>>) -> Bitfield {
        Bitfield {
            pager:      Rc::clone(&pager),
            data:       SparseBitfield::with_pager(Rc::clone(&pager), 0, 1024),
            index:      IndexBitfield::with_bitfield(SparseBitfield::with_pager(Rc::clone(&pager), 1024 + 2048, 256)),
        }
    }

    pub fn new() -> Bitfield {
        let pager = Rc::new(RefCell::new(Pager::new()));
        Bitfield::with_pager(pager)
    }

    pub fn from_buffer(buffer: Vec<u8>) -> Bitfield {
        let pager = Rc::new(RefCell::new(Pager::new()));
        let length = buffer.len();
        let mut offset: usize = 0;
        let mut page_num: usize = 0;
        while offset < length {
            let mut end = offset + PAGE_SIZE;
            if end > length {
                end = length;
            }
            pager.borrow_mut().insert(page_num, buffer[offset..end].to_vec());
            offset += PAGE_SIZE;
            page_num += 1;
        }

        Bitfield::with_pager(pager)
    }

    pub fn get_tree(&self) -> TreeIndex {
        TreeIndex::with_bitfield(SparseBitfield::with_pager(Rc::clone(&self.pager), 1024, 2048))
    }

    pub fn get(&self, index: u64) -> bool {
        self.data.get(index)
    }

    pub fn set(&mut self, index: u64, value: bool) -> bool {
        if !self.data.set(index, value) { return false; }
        let byte = self.data.get_byte(index);
        self.index.set(index, byte)
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

    pub fn to_vec(&self) -> Vec<u8> {
        let pager = self.pager.borrow();
        let mut result: Vec<u8> = vec![0u8; pager.len() * PAGE_SIZE as usize];
        for (&index, value) in pager.iter() {
            let start: usize = index * PAGE_SIZE;
            result[start..(start + PAGE_SIZE)].copy_from_slice(value.as_slice());
        }
        result
    }

    pub fn last_updated(&mut self) -> Option<(usize, Vec<u8>)> {
        let mut pager = self.pager.borrow_mut();
        match pager.last_updated() {
            None        => None,
            Some(num)   => Some((num * PAGE_SIZE, pager.get(num).unwrap().to_vec()))
        }
    }
}
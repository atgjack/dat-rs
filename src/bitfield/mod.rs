use std::cell::RefCell;
use std::rc::Rc;
use std::ops::{Range};

mod pager;
mod sparse;
pub mod index;
pub mod tree;

use bitfield::pager::Pager;
use bitfield::sparse::SparseBitfield;
use bitfield::index::IndexBitfield;
use bitfield::tree::TreeBitfield;

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
            let mut p = pager.borrow_mut();
            let page_size = p.get_page_size();
            let mut end = offset + page_size;
            if end > length {
                end = length;
            }
            p.insert(page_num, buffer[offset..end].to_vec());
            offset += page_size;
            page_num += 1;
        }

        Bitfield::with_pager(pager)
    }

    pub fn get_tree(&self) -> TreeBitfield {
        TreeBitfield::with_bitfield(SparseBitfield::with_pager(Rc::clone(&self.pager), 1024, 2048))
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
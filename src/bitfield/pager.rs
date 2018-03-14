use std::collections::{HashMap};
use std::collections::hash_map::{Iter, Entry};
use indexmap::IndexSet;

const PAGE_SIZE: usize = 3328;

pub struct Pager {
    map:        HashMap<usize, Vec<u8>>,
    updated:    IndexSet<usize>,
    page_size:  usize,
}

impl Pager {
    pub fn new() -> Pager {
        Pager {
            map:        HashMap::new(),
            updated:    IndexSet::new(),
            page_size:  PAGE_SIZE,
        }
    }

    pub fn get(&self, index: usize) -> Option<&Vec<u8>> {
        self.map.get(&index)
    }

    pub fn set(&mut self, page_num: usize, byte_num: usize, value: u8) -> bool {
        match self.map.entry(page_num) {
            Entry::Vacant(page) => {
                if value == 0 { return false; }
                let mut vec = vec![0u8; self.page_size];
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

    pub fn get_page_size(&self) -> usize {
        self.page_size
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
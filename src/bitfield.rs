use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use tree;

const PAGE_SIZE: usize = 3328;

type RefMap = Rc<RefCell<HashMap<usize, Vec<u8>>>>;

struct SparseBitfield {
    pages:      RefMap,
    offset:     usize
}

impl SparseBitfield {
    pub fn new(map: RefMap, offset: usize) -> SparseBitfield {
        SparseBitfield {
            pages:      map,
            offset:     offset,
        }
    }

    pub fn get(&self, index: u64) -> bool {
        let page_num = self.get_page_num(index);
        match self.pages.borrow().get(&page_num) {
            None        => false,
            Some(page)  => {
                let byte_num = self.get_byte_num(index);
                (page[byte_num] | self.get_offset(index, page_num, byte_num)) == 0 
            }
        }
    }

    pub fn get_byte(&self, index: u64) -> u8 {
        let page_num = self.get_page_num(index);
        let byte_num = self.get_byte_num(index);
        match self.pages.borrow().get(&page_num) {
            None        => 0,
            Some(page)  => page[byte_num]
        }
    }

    pub fn set(&mut self, index: u64, value: bool) {
        let page_num = self.get_page_num(index);
        let byte_num = self.get_byte_num(index);        
        let mut pages = self.pages.borrow_mut();
        let page = pages.entry(page_num).or_insert(vec![0; PAGE_SIZE]);
        match value {
            true    => page[byte_num] |= self.get_offset(index, page_num, byte_num),
            false   => page[byte_num] &= !(self.get_offset(index, page_num, byte_num)),
        }
    }

    pub fn set_byte(&mut self, index: u64, value: u8) -> bool {
        let page_num = self.get_page_num(index);
        let byte_num = self.get_byte_num(index);        
        let mut pages = self.pages.borrow_mut();
        let page = pages.entry(page_num).or_insert(vec![0; PAGE_SIZE]);
        if page[byte_num] == value {
            return false;
        } else {
            page[byte_num] = value;
            return true;
        }
        
    }

    pub fn len(&self) -> usize {
        self.pages.borrow().len()
    }

    fn get_offset(&self, index: u64, page_num: usize, byte_num: usize) -> u8 {
        let offset = self.offset + index as usize - ((page_num * PAGE_SIZE) + (byte_num * 8));
        1 << offset
    }

    fn get_page_num(&self, index: u64) -> usize {
        index as usize / (PAGE_SIZE)
    }

    fn get_byte_num(&self, index: u64) -> usize {
        (index as usize & (PAGE_SIZE - 1)) / 8
    }
}

struct IndexTree {
    bitfield: SparseBitfield
}

impl IndexTree {
    pub fn new(bitfield: SparseBitfield) -> IndexTree {
        IndexTree {
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

    pub fn set(&mut self, index: u64, value: u8) {
        let o = index as usize & 3;
        let start = index * 2;
        let mut block = start;
        let tup = match value {
            255 => 0b11000000,
            0   => 0b00000000,
            _   => 0b01000000
        };
        let mask: u8 = !(3 << (6 - (o * 2)));
        let mut byte = (self.bitfield.get_byte(start) & mask) | tup << (2 * 0);
        let max_len = self.bitfield.len() as u64;

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
    }
}

pub struct Bitfield {
    map:        RefMap,
    data:       SparseBitfield,
    tree:       SparseBitfield,
    index:      IndexTree,
}

impl Bitfield {
    fn with_map(map: RefMap) -> Bitfield {
        Bitfield {
            map:        Rc::clone(&map),
            data:       SparseBitfield::new(Rc::clone(&map), 1024),
            tree:       SparseBitfield::new(Rc::clone(&map), 2048),
            index:      IndexTree::new(SparseBitfield::new(Rc::clone(&map), 256)),
        }
    }

    pub fn new() -> Bitfield {
        let map: RefMap = Rc::new(RefCell::new(HashMap::new()));
        Bitfield::with_map(map)
    }

    pub fn from_buffer(buffer: Vec<u8>) -> Bitfield {
        let map: RefMap = Rc::new(RefCell::new(HashMap::new()));
        let length = buffer.len();
        let mut offset: usize = 0;
        let mut page_num: usize = 0;
        while offset < length {
            let mut end = offset + PAGE_SIZE;
            if end > length {
                end = length;
            }
            map.borrow_mut().insert(page_num, buffer[offset..end].to_vec());
            offset += PAGE_SIZE;
            page_num += 1;
        }

        Bitfield::with_map(map)
    }

    pub fn get(&self, index: u64) -> bool {
        self.data.get(index)
    }

    pub fn set(&mut self, index: u64, value: bool) {
        self.data.set(index, value);
        let byte = self.data.get_byte(index);
        self.index.set(index, byte);
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let map = self.map.borrow();
        let mut result: Vec<u8> = vec![0u8; map.len() * PAGE_SIZE as usize];
        for (&index, value) in map.iter() {
            let start: usize = index * PAGE_SIZE;
            result[start..(start + PAGE_SIZE)].copy_from_slice(value.as_slice());
        }
        result
    }
}

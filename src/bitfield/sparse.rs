use std::cell::RefCell;
use std::rc::Rc;
use bitfield::pager::Pager;

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
        let pager = self.pager.borrow();
        pager.len() as u64 * pager.get_page_size() as u64 * 8
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
use bitfield::sparse::SparseBitfield;
use flat;

pub struct IndexBitfield {
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
            let sibling = self.bitfield.get_byte(flat::sibling(current));
            if flat::is_left(current) {
                byte = (self.convert_to_index(byte) << 4) | self.convert_to_index(sibling);
            } else {
                byte = (self.convert_to_index(sibling) << 4) | self.convert_to_index(byte);
            }

            current = flat::parent(current);
        }

        current != start
    }
}

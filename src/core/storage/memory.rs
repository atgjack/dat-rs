use std::io::{Result, Write, Read, Seek, SeekFrom, Cursor};

use core::storage::{Storage, FileType};

pub struct MemoryStorage {
    tree:           Cursor<Vec<u8>>,
    signatures:     Cursor<Vec<u8>>,
    bitfield:       Cursor<Vec<u8>>,
    key:            Cursor<Vec<u8>>,
    secret:         Cursor<Vec<u8>>,
    data:           Cursor<Vec<u8>>,
}

impl MemoryStorage {
    pub fn new() -> MemoryStorage {
        MemoryStorage {
            tree:           Cursor::new(Vec::with_capacity(1024)),
            signatures:     Cursor::new(Vec::with_capacity(1024)),
            bitfield:       Cursor::new(Vec::with_capacity(3328)),
            key:            Cursor::new(Vec::with_capacity(32)),
            secret:         Cursor::new(Vec::with_capacity(64)),
            data:           Cursor::new(Vec::with_capacity(1024)),
        }
    }

    fn get_file(&mut self, file_type: FileType) -> &mut Cursor<Vec<u8>> {
        match file_type {
            FileType::Tree         => &mut self.tree,
            FileType::Signatures   => &mut self.signatures,
            FileType::Bitfield     => &mut self.bitfield,
            FileType::Key          => &mut self.key,
            FileType::Secret       => &mut self.secret,
            FileType::Data         => &mut self.data,
        }
    }
}

impl Storage for MemoryStorage {
    fn init_archive(&mut self, _file_type: FileType) -> bool {
        false
    }

    fn read_archive(&mut self, file_type: FileType, offset: u64, mut buf: &mut [u8]) -> Result<usize> {
        let file = self.get_file(file_type);
        try!(file.seek(SeekFrom::Start(offset)));
        file.read(&mut buf)
    }

    fn write_archive(&mut self, file_type: FileType, offset: u64, buf: &[u8]) -> Result<()> {
        let file = self.get_file(file_type);
        try!(file.seek(SeekFrom::Start(offset)));
        file.write_all(&buf)
    }
}

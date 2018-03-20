use std::io::{Result, Error, ErrorKind};

use core::merkle::Node;
use core::flat;

pub mod file;
pub mod cached;
pub mod memory;

pub use self::file::FileStorage;
pub use self::cached::CachedStorage;
pub use self::memory::MemoryStorage;

#[derive(Clone, Copy, Debug)]
pub enum FileType { Tree, Signatures, Bitfield, Key, Secret, Data }

pub struct StorageState {
    pub bitfield:       Vec<u8>,
    pub key:            Option<[u8; 32]>,
    pub secret:         Option<[u8; 64]>,
}

pub trait Storage {
    fn init_archive(&mut self, file_type: FileType) -> bool;
    fn read_archive(&mut self, file_type: FileType, offset: u64, buf: &mut [u8]) -> Result<usize>;
    fn write_archive(&mut self, file_type: FileType, offset: u64, buf: &[u8]) -> Result<()>;

    fn setup(&mut self) -> Result<()> {
        let file_types = [FileType::Tree, FileType::Signatures, FileType::Bitfield, FileType::Key, FileType::Secret, FileType::Data];

        for &file_type in &file_types {
            if self.init_archive(file_type){
                if let Some(header) = create_header(file_type) {
                    try!(self.write_archive(file_type, 0, &header));
                } 
            }
        }

        Ok(())
    }

    fn get_state(&mut self) -> Result<StorageState> {
        let mut bitfield = Vec::with_capacity(3328);
        let mut buf = [0u8; 3328];
        let mut offset = 32;

        loop {
            match self.read_archive(FileType::Bitfield, offset, &mut buf) {
                Ok(0) | Err(_) => break,
                Ok(num_bytes) => {
                    offset += num_bytes as u64;
                    bitfield.extend_from_slice(&buf[..num_bytes]);
                }
            }
        }

        Ok(StorageState {
            bitfield:   bitfield,
            key:        try!(self.get_key()),
            secret:     try!(self.get_secret()),
        })
    }

    fn get_offset(&mut self, index: u64) -> Result<Option<(u64, u64)>> {
        let block = index;
        let roots = flat::full_roots(block);
        let mut offset = 0;

        for &root in &roots {
            if let Some(root) = try!(self.get_node(root)) {
                offset += root.length;
            }
        }

        if let Some(node) = try!(self.get_node(block)) {
            return Ok(Some((offset, node.length)));
        }

        Ok(None)
    }
    
    fn get_node(&mut self, index: u64) -> Result<Option<Node>> {
        let mut buf = [0u8; 40];

        try!(self.read_archive(FileType::Tree, 32 + 40 * index, &mut buf));
        
        let hash = &buf[..32];
        let mut size: u64 = 0;
        for i in 32..40 {
            size <<= 8;
            size += buf[i] as u64;
        }

        if size == 0 && hash_is_blank(&hash) {
            return Ok(None);
        }

        let node = Node::with_hash(index, &hash, size);
        Ok(Some(node))
    }

    fn put_node(&mut self, index: u64, node: Node) -> Result<()> {
        let mut buf = [0u8; 40];
        let size = node.length;

        buf[..32].copy_from_slice(&node.hash[..32]);
        for i in 0..8 {
            buf[39 - i] = (size >> (8 * i)) as u8; 
        }

        self.write_archive(FileType::Tree, 32 + 40 * index, &buf)
    }

    fn get_roots(&mut self, index: u64) -> Result<Vec<Node>> {
        let roots = flat::full_roots(2 * index);
        let mut result: Vec<Node> = Vec::with_capacity(roots.len());
        
        for &root in &roots {
            if let Some(node) =  try!(self.get_node(root)) {
                result.push(node);
            }
        }

        Ok(result)
    } 

    fn get_data(&mut self, index: u64) -> Result<Option<Vec<u8>>> {
        if let Some((offset, size)) = try!(self.get_offset(index)) {
            let mut buf: Vec<u8> = vec![0u8; size as usize];
            
            try!(self.read_archive(FileType::Data, offset, &mut buf));

            return Ok(Some(buf));
        }

        Ok(None)
    }

    fn put_data(&mut self, index: u64, data: Vec<u8>) -> Result<()> {
        if let Some((offset, size)) = try!(self.get_offset(index)) {
            println!("Writing data");
            
            if data.len() != size as usize {
                return Err(Error::new(ErrorKind::Other, "Unexpected data size."));
            }
            
            return self.write_archive(FileType::Data, offset, &data);
        }

        Ok(())
    }

    fn get_signature(&mut self, index: u64) -> Result<Option<Vec<u8>>> {
        let mut hash: Vec<u8> = Vec::with_capacity(64);
    
        try!(self.read_archive(FileType::Signatures, 32 + 64 * index, &mut hash));
    
        if hash_is_blank(&hash) {
            return Ok(None);
        }

        Ok(Some(hash))
    }

    fn next_signature(&mut self, index: u64) -> Result<Option<Vec<u8>>> {
        match try!(self.get_signature(index)) {
            Some(hash)  => Ok(Some(hash)),
            None        => self.get_signature(index + 1)
        }
    }

    fn put_signature(&mut self, index: u64, hash: Vec<u8>) -> Result<()> {
        self.write_archive(FileType::Signatures, 32 + 64 * index, &hash)
    }

    fn put_bitfield(&mut self, offset: u64, data: Vec<u8>) -> Result<()> {
        self.write_archive(FileType::Bitfield, 32 + offset, &data)
    }

    fn get_key(&mut self) -> Result<Option<[u8; 32]>> {
        let mut buf = [0u8; 32];

        let key_len = try!(self.read_archive(FileType::Key, 0, &mut buf));

        if key_len != buf.len() {
            return Ok(None);
        }

        Ok(Some(buf))
    }

    fn put_key(&mut self, data: [u8; 32]) -> Result<()> {
        self.write_archive(FileType::Key, 0, &data)
    }

    fn get_secret(&mut self) -> Result<Option<[u8; 64]>> {
        let mut buf = [0u8; 64];

        let secret_len = try!(self.read_archive(FileType::Secret, 0, &mut buf));

        if secret_len != buf.len() {
            return Ok(None);
        }

        Ok(Some(buf))
    }
    
    fn put_secret(&mut self, data: [u8; 64]) -> Result<()> {
        self.write_archive(FileType::Secret, 0, &data)
    }
}

fn create_header(file_type: FileType) -> Option<[u8; 32]> {
    let mut result = [0; 32];
    let size: u16;
    let hash: Vec<u8>;
    let magic: u8;

    match file_type {
        FileType::Tree         => {
            size = 72u16;
            hash = b"BLAKE2b".to_vec();
            magic = 2u8;
        },
        FileType::Signatures   => {
            size = 64u16;
            hash = b"Ed25519".to_vec();
            magic = 1u8;
        },
        FileType::Bitfield     => {
            size = 3328u16;
            hash = b"".to_vec();
            magic = 0u8;
        },
        _                       => {
            return None;
        },
    }

    result[0] = 5u8;
    result[1] = 2u8;
    result[2] = 87u8;
    result[3] = magic;

    result[4] = 0u8;

    result[5] = (size >> 8) as u8;
    result[6] = size as u8;

    result[7] = hash.len() as u8;
    result[8..(8 + hash.len())].copy_from_slice(&hash);

    Some(result)
}

fn hash_is_blank(hash: &[u8]) -> bool {
    for i in 0..hash.len() {
        if hash[i] != 0 {
            return false;
        }
    }
    true
}
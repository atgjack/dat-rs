use std::io::{Result, Error, ErrorKind, Write, Read, Seek, SeekFrom};
use std::fs::{File, OpenOptions};
use std::path::{Path};

use lru_cache::LruCache;

use merkle::{Node};
use tree;

enum FileType { Tree, Signatures, Bitfield, Key, Secret, Data }

impl FileType {
    fn filename(&self) -> &str {
        match self {
            &FileType::Tree         => "metadata.tree",
            &FileType::Signatures   => "metadata.signatures",
            &FileType::Bitfield     => "metadata.bitfield",
            &FileType::Key          => "metadata.key",
            &FileType::Secret       => "metadata.secret_key",
            &FileType::Data         => "metadata.data",
        }
    }
    
    fn magic_number(&self) -> Option<u8> {
        match self {
            &FileType::Tree         => Some(2u8),
            &FileType::Signatures   => Some(1u8),
            &FileType::Bitfield     => Some(0u8),
            _                       => None,
        }
    }

    fn entry_size(&self) -> Option<u16> {
        match self {
            &FileType::Tree         => Some(40u16),
            &FileType::Signatures   => Some(64u16),
            &FileType::Bitfield     => Some(3328u16),
            _                       => None,
        }
    }

    fn hash_name(&self) -> Option<Vec<u8>> {
        match self {
            &FileType::Tree         => Some(b"BLAKE2b".to_vec()),
            &FileType::Signatures   => Some(b"Ed25519".to_vec()),
            &FileType::Bitfield     => Some(b"".to_vec()),
            _                       => None,
        }
    }

    fn has_header(&self) -> bool {
        match self {
            &FileType::Tree         => true,
            &FileType::Signatures   => true,
            &FileType::Bitfield     => true,
            _                       => false,
        }
        
    }
}

#[derive(Debug)]
pub struct Storage {
    cache:          LruCache<u64, Node>,
    tree:           File,
    signatures:     File,
    bitfield:       File,
    key:            File,
    secret:         File,
    data:           File,
}

pub struct StorageState {
    pub bitfield:       Vec<u8>,
    pub key:            Option<[u8; 32]>,
    pub secret:         Option<[u8; 64]>,
}

impl Storage {
    pub fn new(path: &Path) -> Result<Storage> {
        if !path.is_dir() {
            return Err(Error::new(ErrorKind::Other, "Path is not a directory"));
        }

        Ok(Storage {
            cache:          LruCache::new(65536),
            tree:           try!(open_or_create(path, FileType::Tree)),
            signatures:     try!(open_or_create(path, FileType::Signatures)),
            bitfield:       try!(open_or_create(path, FileType::Bitfield)),
            key:            try!(open_or_create(path, FileType::Key)),
            secret:         try!(open_or_create(path, FileType::Secret)),
            data:           try!(open_or_create(path, FileType::Data)),
        })
    }

    fn get_offset(&mut self, index: u64) -> Result<Option<(u64, u64)>> {
        let block = index * 2;
        let roots = tree::full_roots(block);
        let mut offset = 0;
        let mut pending = roots.len();

        if pending == 0 {
            return Ok(None)
        }

        for i in 0..roots.len() {
            if let Some(root) = try!(self.get_node(roots[i])) {
                offset += root.length;
                pending -= 1;
                if pending == 0 {
                    break;
                }

                if let Some(node) = try!(self.get_node(block)) {
                    return Ok(Some((offset, node.length)));
                }
            }
        }

        Ok(None)
    }

    pub fn get_state(&mut self) -> Result<StorageState> {
        let mut buf: Vec<u8> = Vec::with_capacity(3328);

        try!(self.bitfield.seek(SeekFrom::Start(32)));
        try!(self.bitfield.read_to_end(&mut buf));

        Ok(StorageState {
            bitfield:   buf,
            key:        try!(self.get_key()),
            secret:     try!(self.get_secret()),
        })
    }

    pub fn get_node(&mut self, index: u64) -> Result<Option<Node>> {
        if let Some(node) = self.cache.get_mut(&index) {
            return Ok(Some(node.clone()));
        }

        let mut buf = [0u8; 40];

        try!(self.tree.seek(SeekFrom::Start(32 + 40 * index)));
        try!(self.tree.read(&mut buf));
        
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
        self.cache.insert(index, node.clone());
        Ok(Some(node))
    }

    pub fn put_node(&mut self, index: u64, node: Node) -> Result<()> {
        let mut buf = [0u8; 40];
        let size = node.length;

        buf[..32].copy_from_slice(&node.hash);
        for i in 0..8 {
            buf[40 - i] = (size >> (8 * i)) as u8; 
        }

        try!(self.tree.seek(SeekFrom::Start(32 + 40 * index)));
        self.tree.write_all(&buf)
    }

    pub fn get_data(&mut self, index: u64) -> Result<Option<Vec<u8>>> {
        if let Some((offset, size)) = try!(self.get_offset(index)) {
            let mut buf: Vec<u8> = Vec::with_capacity(size as usize);
            
            try!(self.data.seek(SeekFrom::Start(offset)));
            try!(self.data.read(&mut buf));

            return Ok(Some(buf));
        }

        Ok(None)
    }

    pub fn put_data(&mut self, index: u64, data: Vec<u8>) -> Result<()> {
        if let Some((offset, size)) = try!(self.get_offset(index)) {
            if data.len() != size as usize {
                return Err(Error::new(ErrorKind::Other, "Unexpected data size."));
            }

            try!(self.data.seek(SeekFrom::Start(offset)));
            return self.data.write_all(&data);
        }

        Ok(())
    }

    pub fn get_signature(&mut self, index: u64) -> Result<Option<Vec<u8>>> {
        let mut hash: Vec<u8> = Vec::with_capacity(64);
    
        try!(self.signatures.seek(SeekFrom::Start(32 + 64 * index)));
        try!(self.signatures.read(&mut hash));
    
        if hash_is_blank(&hash) {
            return Ok(None);
        }

        Ok(Some(hash))
    }

    pub fn next_signature(&mut self, index: u64) -> Result<Option<Vec<u8>>> {
        match try!(self.get_signature(index)) {
            Some(hash)  => Ok(Some(hash)),
            None        => self.get_signature(index + 1)
        }
    }

    pub fn put_signature(&mut self, index: u64, hash: Vec<u8>) -> Result<()> {
        try!(self.signatures.seek(SeekFrom::Start(32 + 64 * index)));
        return self.signatures.write_all(&hash);
    }

    pub fn put_bitfield(&mut self, offset: u64, data: Vec<u8>) -> Result<()> {
            try!(self.bitfield.seek(SeekFrom::Start(32 + offset)));
            return self.bitfield.write_all(&data);
    }

    pub fn get_key(&mut self) -> Result<Option<[u8; 32]>> {
        let mut buf = [0u8; 32];

        try!(self.key.seek(SeekFrom::Start(0)));
        let key_len = try!(self.key.read(&mut buf));

        if key_len != buf.len() {
            return Ok(None);
        }

        Ok(Some(buf))
    }

    pub fn put_key(&mut self, data: [u8; 32]) -> Result<()> {
        try!(self.key.seek(SeekFrom::Start(0)));
        self.key.write_all(&data)        
    }

    pub fn get_secret(&mut self) -> Result<Option<[u8; 64]>> {
        let mut buf = [0u8; 64];

        try!(self.secret.seek(SeekFrom::Start(0)));
        let secret_len = try!(self.secret.read(&mut buf));

        if secret_len != buf.len() {
            return Ok(None);
        }

        Ok(Some(buf))
    }
    
    pub fn put_secret(&mut self, data: [u8; 64]) -> Result<()> {
        try!(self.secret.seek(SeekFrom::Start(0)));
        self.secret.write_all(&data)        
    }
}

fn open_or_create(path: &Path, file_type: FileType) -> Result<File> {
    match OpenOptions::new().write(true).open(path.join(file_type.filename())) {
        Ok(file) => Ok(file),
        Err(_) => create_file(path, file_type)
    }
}

fn create_file(path: &Path, file_type: FileType) -> Result<File> {
    let mut file = match File::create(path.join(file_type.filename())) {
        Ok(file) => file,
        Err(err) => return Err(err),
    };

    if !file_type.has_header() {
        return Ok(file);
    }

    match file.write_all(&header(file_type)){
        Ok(_) => Ok(file),
        Err(err) => Err(err)
    }
}

fn header(file_type: FileType,) -> [u8; 32] {
    let mut result = [0; 32];
    let size = file_type.entry_size().unwrap();
    let hash = file_type.hash_name().unwrap();

    result[0] = 5u8;
    result[1] = 2u8;
    result[2] = 87u8;
    result[3] = file_type.magic_number().unwrap();

    result[4] = 0u8;

    result[5] = (size >> 8) as u8;
    result[6] = size as u8;

    result[7] = hash.len() as u8;
    result[8..(8 + hash.len())].copy_from_slice(&hash);

    result
}

fn hash_is_blank(hash: &[u8]) -> bool {
    for i in 0..hash.len() {
        if hash[i] != 0 {
            return false;
        }
    }
    true
}
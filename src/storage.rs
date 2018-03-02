use std::io::{Result, Error, ErrorKind, Write, Read};
use std::fs::{File, OpenOptions};
use std::path::{Path};

enum FileType { Tree, Signatures, Bitfield, Key, Data }

impl FileType {
    fn filename(&self) -> &str {
        match self {
            &FileType::Tree         => "metadata.tree",
            &FileType::Signatures   => "metadata.signatures",
            &FileType::Bitfield     => "metadata.bitfield",
            &FileType::Key          => "metadata.key",
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
    tree:           File,
    signatures:     File,
    bitfield:       File,
    key:       File,
    data:       File,
}

impl Storage {
    pub fn new(path: &Path) -> Result<Storage> {
        if !path.is_dir() {
            return Err(Error::new(ErrorKind::Other, "Path is not a directory"));
        }

        Ok(Storage {
            tree:           try!(open_or_create(path, FileType::Tree)),
            signatures:     try!(open_or_create(path, FileType::Signatures)),
            bitfield:       try!(open_or_create(path, FileType::Bitfield)),
            key:            try!(open_or_create(path, FileType::Key)),
            data:           try!(open_or_create(path, FileType::Data)),
        })
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
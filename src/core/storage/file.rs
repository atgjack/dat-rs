use std::io::{Result, Error, ErrorKind, Write, Read, Seek, SeekFrom};
use std::fs::{File, OpenOptions, create_dir};
use std::path::{Path};

use core::storage::{Storage, FileType};

pub struct FileStorage {
    tree:           File,
    signatures:     File,
    bitfield:       File,
    key:            File,
    secret:         File,
    data:           File,
}

impl FileStorage {
    pub fn new(path: &Path) -> Result<FileStorage> {
        if !path.is_dir() {
            return Err(Error::new(ErrorKind::Other, "Path is not a directory"));
        }

        let path = &path.join(".dat");
        if let Err(err) = create_dir(&path) {
            if err.kind() != ErrorKind::AlreadyExists {
                return Err(err);
            }
        };

        Ok(FileStorage {
            tree:           try!(open_or_create(path, FileType::Tree)),
            signatures:     try!(open_or_create(path, FileType::Signatures)),
            bitfield:       try!(open_or_create(path, FileType::Bitfield)),
            key:            try!(open_or_create(path, FileType::Key)),
            secret:         try!(open_or_create(path, FileType::Secret)),
            data:           try!(open_or_create(path, FileType::Data)),
        })
    }

    fn get_file(&mut self, file_type: FileType) -> &mut File {
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

impl Storage for FileStorage {
    fn init_archive(&mut self, file_type: FileType) -> bool {
        let file = self.get_file(file_type);
        match file.metadata() {
            Ok(metadata) => metadata.len() > 0,
            Err(_)       => false
        }
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

fn open_or_create(path: &Path, file_type: FileType) -> Result<File> {
    let filename = match file_type {
            FileType::Tree         => "metadata.tree",
            FileType::Signatures   => "metadata.signatures",
            FileType::Bitfield     => "metadata.bitfield",
            FileType::Key          => "metadata.key",
            FileType::Secret       => "metadata.secret_key",
            FileType::Data         => "metadata.data",
    };

    match OpenOptions::new().read(true).write(true).open(path.join(filename)) {
        Ok(file)    => Ok(file),
        Err(_)      => OpenOptions::new().create(true).read(true).write(true).open(path.join(filename))
    }
}
use std::io;
use std::io::{Result};
use std::path::{Path};
use std::ops::{Range};

use rand::OsRng;
use sha2::Sha512;
use blake2::{Blake2b, Digest};
use ed25519_dalek::{Keypair, Signature};

use storage::{Storage};
use merkle::{Tree};
use bitfield::{Bitfield, TreeIndex};

// const LEAF_TYPE : &'static [u8] = &[0];
// const PARENT_TYPE : &'static [u8] = &[1];
const ROOT_TYPE: &'static [u8] = &[2];
// const HYPERCORE: &'static [u8] = b"hypercore";

pub struct Feed {
    storage:    Storage,
    blocks:     u64,
    length:     u64,
    key:        [u8; 32],
    secret:     [u8; 64],
    merkle:     Tree,
    bitfield:   Bitfield,
    tree:       TreeIndex,
}

impl Feed {
    pub fn new(path: &Path) -> Result<Feed> {
        let mut storage = try!(Storage::new(path));
        let state = try!(storage.get_state());
        let mut generate_key = true;
        let mut key = [0u8; 32];
        let mut secret = [0u8; 64];

        // Add discovery_key

        let bitfield = Bitfield::from_buffer(state.bitfield);
        let tree = bitfield.get_tree();
        let blocks = tree.blocks();

        if state.key.is_some() && state.secret.is_some() {
            key = state.key.unwrap();
            secret = state.secret.unwrap();
            let message: &[u8] = b"Verify Me.";
            if let Ok(pair) = Keypair::from_bytes(&[&secret[..32], &key[..]].concat()) {
                let signature: Signature = pair.sign::<Sha512>(message);
                if pair.verify::<Sha512>(message, &signature) {
                    generate_key = false;
                }
            };
        }

        if generate_key {
            let mut cspring: OsRng = try!(OsRng::new());
            let pair: Keypair = Keypair::generate::<Sha512>(&mut cspring);
            key = pair.public.to_bytes();
            secret[32..].copy_from_slice(&key);
            secret[..32].copy_from_slice(&pair.secret.to_bytes());
            try!(storage.put_key(key));
            try!(storage.put_secret(secret));
        }

        let roots = try!(storage.get_roots(blocks));
        let merkle = Tree::with_roots(roots.clone());
        let length = roots.into_iter().fold(0, |sum, root| root.length + sum);

        Ok(Feed {
            storage:    storage,
            blocks:     blocks,
            length:     length,
            key:        key,
            secret:     secret,
            merkle:     merkle,
            bitfield:   bitfield,
            tree:       tree,
        })
    }

    pub fn has(&self, index: u64) -> bool {
        self.bitfield.get(index)
    }

    pub fn has_range(&self, range: Range<u64>) -> bool {
        range.end - range.start == self.bitfield.total(range)
    }

    pub fn downloaded(&self, range: Range<u64>) -> u64 {
        self.bitfield.total(range)
    }

    // pub fn get(&mut self, index: u64) -> DataFuture {
    //     if !self.bitfield.get(index) {

    //     }
    //     match self.storage.get_data(index) {
    //         Ok(data) => future::ok(data),
    //         Err(err) => future::err(err)
    //     }
    // }

        pub fn get(&mut self, index: u64) -> Result<Option<Vec<u8>>> {
            if !self.bitfield.get(index) {
                return Err(io::Error::new(io::ErrorKind::Other, "Index not found."));
            }
            self.storage.get_data(index * 2)
        }

    // pub fn head(&mut self) -> DataFuture {
    //     let len = self.length;
    //     if len == 0 { return future::ok(None); }
    //     self.get(len - 1)
    // }

    // pub fn download(&mut self, range: Range<u64>) -> DataFuture {

    // }

    // fn fetch(&self, index: u64) {

    // }

    fn sign_roots(&mut self) -> Result<()> {
        let mut hasher = Blake2b::new();
        hasher.input(ROOT_TYPE);

        for root in &self.merkle.roots {
            hasher.input(&root.hash);
            hasher.input(&encodebe(root.index));
            hasher.input(&encodebe(root.length));
        }

        let signature = match Keypair::from_bytes(&[&self.secret[..32], &self.key[..]].concat()) {
            Ok(pair)    => pair.sign::<Sha512>(&hasher.result()),
            Err(_)      => return Err(io::Error::new(io::ErrorKind::Other, "Unable to sign roots.")),
        };

        self.storage.put_signature(self.blocks, signature.to_bytes().to_vec())
    }

    pub fn append(&mut self, data: Vec<u8>) -> Result<()> {
        let len = data.len();
        if len == 0 { return Ok(()); }

        let nodes = self.merkle.insert::<Blake2b>(data);

        for node in nodes {
            if let Err(_) = self.storage.put_node(node.index, node.clone()) {
                return Err(io::Error::new(io::ErrorKind::Other, "Unable to save node."))
            }
            if let Some(data) = node.data {
                if let Err(_) = self.storage.put_data(node.index, data) {
                    return Err(io::Error::new(io::ErrorKind::Other, "Unable to save data."))
                }
            }
        }

        self.bitfield.set(self.blocks, true);
        self.tree.set(self.blocks);
        while let Some((offset, data)) = self.bitfield.last_updated() {
                try!(self.storage.put_bitfield(offset as u64, data));
        }
        self.length += len as u64;
        self.blocks += 1;

        self.sign_roots()
    }
}

fn encodebe(input: u64) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::with_capacity(8);
    for i in 0..8 {
        result.push((input >> (8 * i)) as u8);
    }
    result
}


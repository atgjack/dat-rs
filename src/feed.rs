use std::io::{Result};
use std::path::{Path};

use rand::OsRng;
use sha2::Sha512;
use ed25519_dalek::{Keypair, Signature};

use storage::{Storage};
use merkle::{Tree};
use bitfield::{Bitfield, TreeIndex};

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
            if let Ok(pair) = Keypair::from_bytes(&[&key[..], &secret[..]].concat()) {
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
            secret[..32].copy_from_slice(&key);
            secret[32..].copy_from_slice(&pair.secret.to_bytes());
            try!(storage.put_key(key));
            try!(storage.put_secret(secret));
        }

        let roots = try!(storage.get_roots(blocks));
        let merkle = Tree::with_roots(roots.clone());
        let length = roots.into_iter()
                          .fold(0, |sum, root| root.length + sum);

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
}


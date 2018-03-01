use digest::Digest;
use generic_array::GenericArray;
use generic_array::typenum::U64;

use tree;

#[derive(Debug)]
pub struct Node {
    index:  u64,
    parent: u64,
    length: u64,
    data:   Option<Vec<u8>>,
    hash:   GenericArray<u8, U64>,
}

impl Node {
    pub fn with_data<D>(idx: u64, data: Vec<u8>) -> Node
                        where D: Digest<OutputSize=U64> {
        let mut hasher = D::new();
        hasher.input(&data);
        Node {
            index:      idx,
            parent:     tree::parent(idx),
            length:     data.len() as u64,
            data:       Some(data),
            hash:       hasher.result(),
        }
    }

    pub fn with_nodes<D>(left: &Node, right: &Node) -> Node
                        where D: Digest<OutputSize=U64> {
        let mut hasher = D::new();
        hasher.input(&left.hash);
        hasher.input(&right.hash);
        Node {
            index:      left.parent,
            parent:     tree::parent(left.parent),
            length:     left.length + right.length,
            data:       None,
            hash:       hasher.result(),
        }
    }
}

#[derive(Debug)]
pub struct Tree {
    pub roots:      Vec<Node>,
    blocks:     u64,
}

impl Tree {
    pub fn new() -> Tree {
        Tree {
            roots:      Vec::with_capacity(2),
            blocks:     0,
        }
    }

    pub fn insert<D>(&mut self, data: Vec<u8>) 
                    where D: Digest<OutputSize=U64> {
        let node = Node::with_data::<D>(self.blocks * 2, data);
        self.blocks += 1;
        self.roots.push(node);

        while self.roots.len() > 1 {
            let right = self.roots.pop().unwrap();
            let left = self.roots.pop().unwrap();

            if left.parent != right.parent {
                self.roots.push(left);
                self.roots.push(right);
                break;
            }

            let parent = Node::with_nodes::<D>(&left, &right);
            self.roots.push(parent);
        }
    }
}
use digest::{Digest, VariableOutput};

use common::flat;

#[derive(Debug, Clone)]
pub struct Node {
    pub index:  u64,
    pub parent: u64,
    pub length: u64,
    pub data:   Option<Vec<u8>>,
    pub hash:   [u8; 32],
}

impl Node {
    pub fn with_hash(idx: u64, hash: &[u8], length: u64) -> Node {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(hash);
        Node {
            index:      idx,
            parent:     flat::parent(idx),
            length:     length,
            data:       None,
            hash:       arr,
        }
    }

    pub fn with_data<D>(idx: u64, data: Vec<u8>) -> Node
                        where D: VariableOutput + Digest {
        let mut arr = [0u8; 32];
        let mut hasher: D = VariableOutput::new(32).unwrap();
        hasher.input(&data);
        hasher.variable_result(&mut arr).unwrap();
        Node {
            index:      idx,
            parent:     flat::parent(idx),
            length:     data.len() as u64,
            data:       Some(data),
            hash:       arr,
        }
    }

    pub fn with_nodes<D>(left: &Node, right: &Node) -> Node
                        where D: VariableOutput + Digest {
        let mut arr = [0u8; 32];
        let mut hasher: D = VariableOutput::new(32).unwrap();
        hasher.input(&left.hash);
        hasher.input(&right.hash);
        hasher.variable_result(&mut arr).unwrap();
        Node {
            index:      left.parent,
            parent:     flat::parent(left.parent),
            length:     left.length + right.length,
            data:       None,
            hash:       arr,
        }
    }
}

#[derive(Debug)]
pub struct Tree {
    pub roots:      Vec<Node>,
    pub blocks:     u64,
}

impl Tree {
    pub fn new() -> Tree {
        Tree {
            roots:      Vec::with_capacity(2),
            blocks:     0,
        }
    }

    pub fn with_roots(roots: Vec<Node>) -> Tree {
        let blocks = match roots.last() {
            Some(last)  => 1 + flat::right_span(last.index) / 2,
            None        => 0,
        };

        Tree {
            roots:      roots,
            blocks:     blocks,
        }
    }
    
    pub fn insert<D>(&mut self, data: Vec<u8>) -> Vec<Node>
                    where D: VariableOutput + Digest {
        let mut nodes: Vec<Node> = Vec::new();
        let node = Node::with_data::<D>(self.blocks * 2, data);
        self.blocks += 1;
        self.roots.push(node.clone());

        nodes.push(node.clone());

        while self.roots.len() > 1 {
            let right = self.roots.pop().unwrap();
            let left = self.roots.pop().unwrap();

            if left.parent != right.parent {
                self.roots.push(left);
                self.roots.push(right);
                break;
            }

            let parent = Node::with_nodes::<D>(&left, &right);
            self.roots.push(parent.clone());
            nodes.push(parent)
        }

        nodes
    }
}
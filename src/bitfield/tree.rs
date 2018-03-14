use bitfield::sparse::SparseBitfield;
use flat;

pub struct TreeBitfield {
    bitfield: SparseBitfield
}

impl TreeBitfield {
    pub fn new() -> TreeBitfield {
        TreeBitfield {
            bitfield:   SparseBitfield::new()
        }
    }

    pub fn with_bitfield(bitfield: SparseBitfield) -> TreeBitfield {
        TreeBitfield {
            bitfield: bitfield
        }
    }

    pub fn get(&self, index: u64) -> bool {
        self.bitfield.get(index)
    }

    pub fn set(&mut self, index: u64) -> bool {
        let mut current = index * 2;
        if !self.bitfield.set(current, true) { return false; }
        while self.bitfield.get(flat::sibling(current)) {
            current = flat::parent(current);
            if !self.bitfield.set(current, true) { break; }
        }
        true
    }

    pub fn verfied_by(&self, index: u64) -> Option<u64> {
        if !self.get(index) { return None; }
        let mut depth = flat::depth(index);
        let mut top = index;
        let mut parent = flat::parent_with_depth(index, depth);
        depth += 1;

        // Find current root.
        while self.get(parent) && self.get(flat::sibling(top)) {
            top = parent;
            parent = flat::parent_with_depth(top, depth);
            depth += 1;
        }

        // Extend right down.
        depth -= 1;
        while depth > 0 {
            parent = flat::index(depth, flat::offset_with_depth(top, depth) + 1);
            top = match flat::child_with_depth(parent, depth, true) {
                Some(child) => child,
                None => return None,   
            };
            depth -= 1;

            while !self.get(top) && depth > 0 { 
                top = match flat::child_with_depth(top, depth, true) {
                    Some(child) => child,
                    None => return None,   
                };
                depth -= 1;
            }
        }

        match self.get(top) {
            true    => Some(top + 2),
            false   => Some(top),
        }
    }

    pub fn blocks(&self) -> u64 {
        let mut top = 0;
        let mut next = 0;
        let max = self.bitfield.len();

        while flat::right_span(next) < max {
            next = flat::parent(next);
            if self.get(next) {
                top = next;
            }
        }

        if !self.get(top) {
            return 0;
        }
        

        match self.verfied_by(top) {
            Some(val) => val / 2,
            None      => 0,
        }
    }

    pub fn roots(&self) -> Vec<u64> {
        flat::full_roots(2 * self.blocks())
    }

    pub fn digest(&self, index: u64) -> u64 {
        if self.get(index) {
            return 1;
        }

        let mut digest = 0u64;
        let mut next = flat::sibling(index);
        let max = match next + 2 > self.bitfield.len() {
            true    => next + 2,
            false   => self.bitfield.len()
        };

        let mut bit = 2u64;
        let mut depth = flat::depth(index);
        let mut parent = flat::parent_with_depth(next, depth);
        depth += 1;

        while flat::right_span(next) < max || flat::left_span(parent) > 0 {
            if self.get(next) {
                digest |= bit;
            }

            if self.get(parent) {
                digest |= 2 * bit + 1;
                if digest + 1 == 4 * bit {
                    return 1;
                }
                return digest;
            }

            next = flat::sibling(parent);
            parent = flat::parent_with_depth(next, depth);
            depth += 1;
            bit *= 2;
        }

        digest
    }

    pub fn proof(&self, index: u64, opts: ProofOpts) -> Option<(Vec<u64>, u64)> {
        if !self.get(index) { return None; }
        
        let mut nodes: Vec<u64> = Vec::new();
        let mut digest = opts.digest;
        let mut remote = opts.remote;
        
        if opts.hash { nodes.push(index); }
        if digest == 1 { return Some((nodes, 0)); }

        let mut roots: Vec<u64>;
        let mut sibling;
        let mut next = index;
        
        digest >>= 1;
        while digest > 0 {
            if digest == 1 && digest & 1 != 0 {
                if self.get(next) { remote.set(next); }
                if flat::sibling(next) < next { next = flat::sibling(next); }
                roots = flat::full_roots(flat::right_span(next) + 2);
                for &root in &roots {
                    if self.get(root) { remote.set(root); }
                }
                break;
            }

            sibling = flat::sibling(next);
            if digest & 1 > 0 && self.get(sibling) { remote.set(sibling); }
            next = flat::parent(next);
            digest >>= 1;
        }

        next = index;

        while !remote.get(next) {
            sibling = flat::sibling(next);
            if !self.get(sibling) {
                match self.verfied_by(next) {
                    None => return None,
                    Some(val) => {
                        roots = flat::full_roots(val);
                        for &root in &roots {
                            if root != next && !remote.get(root) { nodes.push(root); }
                        }
                        return Some((nodes, val));
                    }
                }
            } else {
                if !remote.get(sibling) { nodes.push(sibling); }
            }

            next = flat::parent(next);
        }

        Some((nodes, 0))
    }
}

pub struct ProofOpts {
    remote:     TreeBitfield,
    digest:     u64,
    hash:       bool,
}

impl ProofOpts {
    pub fn new() -> ProofOpts {
        ProofOpts {
            remote:     TreeBitfield::new(),
            digest:     0,
            hash:       false,
        }
    }

    pub fn set_remote(&mut self, remote: TreeBitfield) {
        self.remote = remote;
    }

    pub fn set_digest(&mut self, digest: u64) {
        self.digest = digest;
    }

    pub fn set_hash(&mut self, hash: bool) {
        self.hash = hash;
    }
}
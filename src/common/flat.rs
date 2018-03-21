// Utilites

pub fn offset_with_depth(idx: u64, depth: u64) -> u64 {
    idx >> (depth + 1)
}

pub fn parent_with_depth(idx: u64, depth: u64) -> u64 {
    index(depth + 1, offset_with_depth(idx, depth) >> 1)
}

pub fn sibling_with_depth(idx: u64, depth: u64) -> u64 {
    index(depth, offset(idx) ^ 1)
}

pub fn child_with_depth(idx: u64, depth: u64, is_left: bool) -> Option<u64>  {
    if depth == 0 || idx & 1 == 0 {
        return None;
    }

    let adder = if is_left { 0 } else { 1 };
    Some(index(depth - 1, (offset_with_depth(idx, depth) * 2) + adder))
}

pub fn span_with_depth(idx: u64, depth: u64, is_left: bool) -> u64 {
    if depth == 0  { 
        return idx; 
    }
    
    let adder = if is_left { 0 } else { 1 };
    (offset_with_depth(idx, depth) + adder) * (2 << depth) - (2 * adder)
}

// Public Functions

pub fn index(depth: u64, offset: u64) -> u64 {
    offset << (depth + 1) | ((1 << depth) - 1)
}

pub fn depth(idx: u64) -> u64 {
    let mut idx = idx;
    let mut depth: u64 = 0;
    while idx & 1 != 0 {
        idx >>= 1;
        depth += 1;
    }
    depth
}

pub fn offset(idx: u64) -> u64 {
    offset_with_depth(idx, depth(idx))
}

pub fn parent(idx: u64) -> u64 {
    parent_with_depth(idx, depth(idx))
}

pub fn sibling(idx: u64) -> u64 {
    sibling_with_depth(idx, depth(idx))
}

pub fn uncle(idx: u64) -> u64 {
    let depth = depth(idx);
    sibling_with_depth(parent_with_depth(idx, depth), depth + 1)
}

pub fn left_child(idx: u64) -> Option<u64> {
    child_with_depth(idx, depth(idx), true)
}

pub fn right_child(idx: u64) -> Option<u64>  {
    child_with_depth(idx, depth(idx), false)
}

pub fn children(idx: u64) -> Option<[u64; 2]> {
    let depth = depth(idx);
    
    if depth == 0 {
        return None;
    }

    let offset = offset_with_depth(idx, depth) << 1;
    Some([index(depth - 1, offset), index(depth - 1, offset + 1)])
}

pub fn left_span(idx: u64) -> u64 {
    span_with_depth(idx, depth(idx), true)
}

pub fn right_span(idx: u64) -> u64  {
    span_with_depth(idx, depth(idx), false)
}

pub fn spans(idx: u64) -> [u64; 2] {
    let depth = depth(idx);
    [span_with_depth(idx, depth, true), span_with_depth(idx, depth, false)]
}

pub fn count(idx: u64) -> u64 {
    (2 << depth(idx)) - 1
}

pub fn full_roots(idx: u64) -> Vec<u64> {
    let mut vec: Vec<u64> = Vec::with_capacity(64);

    if idx & 1 != 0 {
        vec.push(idx);
        return vec;
    }

    let mut temp: u64 = idx >> 1;
    let mut offset: u64 = 0;
    let mut factor: u64 = 1;

    while temp != 0 {
        while factor << 1 <= temp {
            factor <<= 1;
        }
        vec.push(offset + factor - 1);
        offset += factor << 1;
        temp -= factor;
        factor = 1;
    }

    vec
}

pub fn is_left(idx: u64) -> bool {
    offset(idx) & 1 == 0
}

pub fn is_right(idx: u64) -> bool {
    !is_left(idx)
}
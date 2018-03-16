pub fn length(val: usize) -> usize {
    let mut n = 1 << 7;
    let mut count = 1;
    while val > n && count < 10 {
        count += 1;
        n <<= 7;
    }

    count
}

pub fn encode(val: usize) -> Vec<u8> {
    let mut num = val;
    let mut result: Vec<u8> = Vec::with_capacity(length(val));
    
    while num >> 7 != 0 {
        result.push((num as u8) | 0x80);
        num >>= 7;
    }

    result.push(num as u8);

    result
}

pub fn decode(val: &[u8]) -> usize {
    let mut result = 0usize;

    for i in 0..val.len() {
        let byte = val[i];
        result += (byte as usize & 0x7F) << (i * 7);
        if byte & 0x80 == 0 { break; }
    }

    result
}
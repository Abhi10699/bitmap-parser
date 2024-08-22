pub fn combine_8bits(buffer: &[u8]) -> u32 {
    let mut combined_bits = 0;
    let mut counter = 0;

    for byte in buffer {
        combined_bits = combined_bits as u32 | (*byte as u32) << counter;
        counter = counter + 8;
    }

    combined_bits
}

pub fn combine_8bits_signed(buffer: &[u8]) -> i32 {
    let mut combined_bits = 0;
    let mut counter = 0;

    for byte in buffer {
        combined_bits = combined_bits as i32 | (*byte as i32) << counter;
        counter = counter + 8;
    }

    combined_bits
}

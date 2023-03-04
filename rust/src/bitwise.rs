/// Returns a number as a four bits array in little endian.
pub fn as_u8_array_le(num: u32) -> [u8; 4] {
    [
        num as u8,
        (num >> 8) as u8,
        (num >> 16) as u8,
        (num >> 24) as u8,
    ]
}

/// Returns a number as a four bits array in big endian.
pub fn as_u8_array_be(num: u32) -> [u8; 4] {
    [
        (num >> 24) as u8,
        (num >> 16) as u8,
        (num >> 8) as u8,
        num as u8,
    ]
}

/// Returns a four bits array as a number in little endian.
pub fn as_u32_le(array: &[u8; 4]) -> u32 {
    (array[0] as u32)
        + ((array[1] as u32) << 8)
        + ((array[2] as u32) << 16)
        + ((array[3] as u32) << 24)
}

/// Returns a four bits array as a number in big endian.
pub fn as_u32_be(array: &[u8; 4]) -> u32 {
    ((array[0] as u32) << 24)
        + ((array[1] as u32) << 16)
        + ((array[2] as u32) << 8)
        + (array[3] as u32)
}

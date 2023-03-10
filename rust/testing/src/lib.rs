// Utilities used in tests and examples

pub fn create_binary_content(data: &Vec<u32>, le: bool) -> Vec<u8> {
    let mut buf: Vec<u8> = vec![];
    let bytes_reader = match le {
        true => u32::to_le_bytes,
        false => u32::to_be_bytes,
    };
    for d in data {
        buf.extend(bytes_reader(*d));
    }
    buf
}

pub fn create_corrupted_binary_content(
    data: &Vec<u32>,
    le: bool,
    additional_bytes: &Vec<u8>,
) -> Vec<u8> {
    let mut buf = create_binary_content(data, le);
    buf.extend(additional_bytes);
    buf
}

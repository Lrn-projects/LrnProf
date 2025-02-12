/// The function `debug_binary_format` in Rust prints the hexadecimal representation of the first 4
/// bytes of a given vector as a u64 integer.
///
/// Arguments:
///
/// * `vec`: I see that you have a function `debug_binary_format` that takes a vector of unsigned 8-bit
/// integers as input. The function seems to be trying to interpret the first 4 bytes of the input
/// vector as a little-endian u64 integer and then print it in hexadecimal format.
#[allow(dead_code)]
pub fn debug_binary_format(vec: Vec<u8>) {
    // buffer to easily check the binary format
    let mut array = [0u8; 8];
    for i in vec[0..4].iter() {
        array[..4].copy_from_slice(&vec[0..4]);
    }
    println!("[DEBUG] {:#x}", u64::from_le_bytes(array));
}

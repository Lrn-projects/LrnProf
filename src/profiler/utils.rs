use std::result;

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

pub fn read_addr(task: u32, offset: u64, sizecmds: u32) -> Vec<u8> {
    let size: mach2::vm_types::mach_vm_size_t = sizecmds as mach2::vm_types::mach_vm_size_t;
    // read base addr content
    let mut data = vec![0u8; size as usize]; // allocate buffer
    let mut dataCnt: mach2::message::mach_msg_type_number_t = 0;
    unsafe {
        let result = mach2::vm::mach_vm_read(
            task,
            offset,
            size,
            data.as_mut_ptr() as *mut _,
            &mut dataCnt,
        );
        if result != mach2::kern_return::KERN_SUCCESS {
            eprintln!("Error reading memory: {}", result);
        }
    }
    return data.to_vec();
}

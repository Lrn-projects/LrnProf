use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

// use bincode::{self};

const MAGIC_ARM64: u64 = 0xfeedfacf;

//structure of the Mach-O (Mach object) file format in 64 bits
#[derive(Debug)]
struct MachOBinary {
    header: MachOHeader,
}

#[derive(Debug)]
struct MachOHeader {
    format: u32,
}

struct MachOLoadCommands {
    symbolTable: String,
    dynamicSymbolTable: String,
}

use nom::bytes;

use crate::{logs, utils};

pub fn parse_bin(pid: i32) {
    let output = utils::get_bin_path(pid);
    if !Path::new(&output).exists() {
        logs::error_log("Cannot find the binary of the process".to_string());
    }
    logs::info_log("Binary found".to_string());
    let my_buf = BufReader::new(File::open(output).unwrap());
    let mut bytes_vec: Vec<u8> = Vec::new();
    for byte_or_error in my_buf.bytes() {
        let byte = byte_or_error.unwrap();
        bytes_vec.push(byte);
    }
    // read the magic number of the binary to find the format
    // match the binary format in little endian
    let s: MachOBinary = unsafe { std::ptr::read(bytes_vec.as_ptr() as *const _) };
    if s.header.format == 0xfeedfacf || s.header.format == 0xfeedface {
        logs::info_log("Binary format is Mach-O".to_string());
    }
    // let decoded: Result<String, _> = bincode::deserialize(&bytes_vec);
    // match decoded {
    //     Ok(data) => println!("{:?}", data),
    //     Err(e) => println!("Deserialization error: {:?}", e),
    // }
}

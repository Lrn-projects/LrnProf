// binary file parser
// iterate over the binary straightforward to find the intended data
// https://github.com/aidansteele/osx-abi-macho-file-format-reference

use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

//struct of the Mach-O (Mach object) file magic in 64 bits
#[derive(Debug)]
struct MachOBinary {
    loadCommand: Vec<MachOLoadCommands>,
}

// struct of the Mach-O header
#[derive(Debug)]
#[allow(dead_code)]
struct MachOHeader {
    magic: u32,
    cpuType: u32,
    cpuSubType: u32,
    filetype: u32,
    ncmds: u32,
    sizeofcmds: u32,
    flags: u32,
    reserved: u32,
}

// struct of the Mach-O Load Commands
#[derive(Debug, Clone, Copy)]
struct MachOLoadCommands {
    cmd: u32,
    cmdsize: u32,
}

#[allow(dead_code)]
struct SegmentCommands {
    cmd: u32,
    cmdsize: u32,
    segname: char,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
struct MachOLCSymtabCommand {
    cmd: u32,
    cmdsize: u32,
    symoff: u32,
    nsyms: u32,
    stroff: u32,
    strsize: u32,
}

use crate::{logs, utils};

pub fn parse_bin(pid: i32) {
    let output = utils::get_bin_path(pid);
    if !Path::new(&output).exists() {
        logs::error_log("Cannot find the binary of the process".to_string());
    }
    logs::info_log("Binary found".to_string());
    let my_buf = BufReader::new(File::open(output).unwrap());
    // vector containing the whole binary
    let mut bytes_vec: Vec<u8> = Vec::new();
    for byte_or_error in my_buf.bytes() {
        let byte = byte_or_error.unwrap();
        bytes_vec.push(byte);
    }
    // read the header size and return the size in octets
    let header_size = std::mem::size_of::<MachOHeader>();
    // fetch only the header from the whole binary
    let header_bytes = &bytes_vec[..header_size];
    // convert into the MachOHeader struct
    let header: MachOHeader = unsafe { std::ptr::read(header_bytes.as_ptr() as *const _) };

    // read the magic number of the binary to find the magic
    // match the binary magic in little endian
    if header.magic == 0xfeedfacf || header.magic == 0xfeedface {
        logs::info_log("Binary magic is Mach-O".to_string());
        // get the total size of load command
        let load_commands_size = header.sizeofcmds as usize;
        // extract bytes from load commands
        // get only the load commands from the bytes
        let load_commands_bytes = &bytes_vec[header_size..header_size + load_commands_size];
        // will contain all the load_commands
        let mut load_commands = Vec::new();
        // init the offset to iter over the load_commands
        let mut offset = 0;
        // store all the offset of the binary when iter over the load_commands_size
        let mut offset_map = Vec::new();
        // loop to read all the load_commands
        while offset < load_commands_size {
            // Unsafe operation: Direct memory reading without validity checks.
            //
            // We get a pointer to the bytes starting at `offset` within `load_commands_bytes`.
            // `as_ptr()` gives a `*const u8`, which we cast to `*const MachOLoadCommands`
            // to tell the compiler: "These bytes represent a MachOLoadCommands structure."
            //
            // Then, `std::ptr::read(...)` reads these bytes and interprets them as a `MachOLoadCommands`.
            // If the bytes do not exactly match a `MachOLoadCommands` structure, this leads to **Undefined Behavior**.
            //
            // Safer alternative: Check that `offset + size_of::<MachOLoadCommands>() <= load_commands_bytes.len()`
            // before performing this conversion.
            let cmd: MachOLoadCommands =
                unsafe { std::ptr::read(load_commands_bytes[offset..].as_ptr() as *const _) };
            // get the size of the current load_command
            let cmdsize = cmd.cmdsize;
            // add the current load_command to the vector
            load_commands.push(cmd);
            // push the current offset to the map
            offset_map.push((cmd.cmd, offset));
            // move the offset forward to get the next load_command
            offset += cmdsize as usize;
        }
        let s = MachOBinary {
            loadCommand: load_commands,
        };
        // iterate over the struct load_command
        for i in s.loadCommand {
            // found the symbols table
            if i.cmd == 2 {
                println!("Found load command with cmd value 2");
                // get the offset_map index matching the symtab int
                let lc_symtab_offset_index = offset_map.iter().position(|x| x.0 == 2);
                // get the value corresponding to the index
                let lc_symtab_offset = offset_map[6].1;
                // cast the MachOLCSymtabCommand struct from the load_commands_bytes vector using the offset index
                let symtab_cmd: MachOLCSymtabCommand = unsafe {
                    std::ptr::read(load_commands_bytes[lc_symtab_offset..].as_ptr() as *const _)
                };
                println!("offset: {:?}", symtab_cmd);
                break;
            }
        }
    }
    // create a struct containing all the load_command
}
// let decoded: Result<String, _> = bincode::deserialize(&bytes_vec);
// match decoded {
//     Ok(data) => println!("{:?}", data),
//     Err(e) => println!("Deserialization error: {:?}", e),
// }

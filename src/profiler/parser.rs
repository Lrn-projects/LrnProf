// binary file parser
// iterate over the binary straightforward to find the intended data
// https://github.com/aidansteele/osx-abi-macho-file-format-reference

use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

#[derive(Debug)]
struct MachOBinary {
    loadCommand: Vec<LoadCommand>,
}

#[derive(Debug)]
#[allow(dead_code)]
struct MachHeader64 {
    magic: u32,
    cpuType: u32,
    cpuSubType: u32,
    filetype: u32,
    ncmds: u32,
    sizeofcmds: u32,
    flags: u32,
    reserved: u32,
}

#[derive(Debug, Clone, Copy)]
struct LoadCommand {
    cmd: u32,
    cmdsize: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
struct SymtabCommand {
    cmd: u32,
    cmdsize: u32,
    // symbol table offset
    symoff: u32,
    nsyms: u32,
    // string table offset
    stroff: u32,
    strsize: u32,
}

#[derive(Debug)]
// structure for one entry of the symtab
struct Nlist64 {
    n_strx: u32,
    n_type: u8,
    n_sect: u8,
    n_desc: u16,
    n_value: u64,
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
    let header_size = std::mem::size_of::<MachHeader64>();
    // fetch only the header from the whole binary
    let header_bytes = &bytes_vec[..header_size];
    // convert into the MachHeader64 struct
    let header: MachHeader64 = unsafe { std::ptr::read(header_bytes.as_ptr() as *const _) };

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
            // `as_ptr()` gives a `*const u8`, which we cast to `*const LoadCommand`
            // to tell the compiler: "These bytes represent a LoadCommand structure."
            //
            // Then, `std::ptr::read(...)` reads these bytes and interprets them as a `LoadCommand`.
            // If the bytes do not exactly match a `LoadCommand` structure, this leads to **Undefined Behavior**.
            //
            // Safer alternative: Check that `offset + size_of::<LoadCommand>() <= load_commands_bytes.len()`
            // before performing this conversion.
            let cmd: LoadCommand =
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
                // cast the SymtabCommand struct from the load_commands_bytes vector using the offset index
                let symtab_cmd: SymtabCommand = unsafe {
                    std::ptr::read(load_commands_bytes[lc_symtab_offset..].as_ptr() as *const _)
                };
                // loop over the all symtab to get all entries
                for i in 0..symtab_cmd.nsyms {
                    // offset of one symtab entry
                    let symbol_offset =
                        symtab_cmd.symoff + (i * std::mem::size_of::<Nlist64>() as u32);
                    // one symtab entry
                    let symtab: Nlist64 = unsafe {
                        std::ptr::read(bytes_vec[symbol_offset as usize..].as_ptr() as *const _)
                    };
                    println!("{:?}", symtab);
                }
                let mut symbol_resolve_map: Vec<(u32, usize)> = Vec::new();
                // loop over the string table and resolve each symbols
                for i in &bytes_vec
                    [symtab_cmd.stroff as usize..(symtab_cmd.stroff + symtab_cmd.strsize) as usize]
                {
                    // let strx = symtab.n_strx as usize;
                    // for i in
                    // symbol_resolve_map.push(());
                }
                // println!("offset: {:?}", string_table);
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

#![allow(unused_variables)]
#![allow(non_snake_case)]

use core::panic;

use crate::logs;
use libc::exit;
use mach2::message::mach_msg_type_number_t;
use mach2::traps::mach_task_self;
use mach2::traps::task_for_pid;
use read_process_memory::*;
embed_plist::embed_info_plist!("../../Info.plist");
mod parser;

pub fn run_profiler(pid: &i32) {
    logs::rp_log("Start running the profiler...");
    let mut task: u32 = 0;

    // kernel error code
    let kernel_success: i32 = 0;

    // threads mutable var
    let mut thread_list: *mut u32 = std::ptr::null_mut();
    let mut thread_count: u32 = 0;
    // buffer for the thead_info output
    let mut thread_info_out: [i32; 1024] = [0; 1024];
    let mut thread_info_out_cnt: u32 = 1024;
    // used for the thread_get_state
    let mut new_state: [u64; 129] = [0; 129];
    let mut new_state_count: u32 = 129;
    // backtrace

    // buffers
    let mut process_address_bytes_buffer = [0; 128];
    let mut next_fp_bytes_vec = [0u8; 8];
    let mut next_lr_bytes_vec = [0u8; 8];

    // used to read the process memory address
    let array: [u64; 128] = [0; 128];
    let size: usize = array.len();

    // flavor
    let thread_basic_info: u32 = 3;
    let thread_id_info: u32 = 4;
    let thread_extended_info: u32 = 5;

    //unwind loop
    let mut addresses: Vec<u64> = Vec::new();

    // binary based address
    #[allow(unused_assignments)]
    let mut bin_loaded_addr: u64 = 0;

    //callback
    let cb = |symbol: &backtrace::Symbol| {
        println!(
            "{:?}",
            symbol
                .name()
                .unwrap_or_else(|| backtrace::SymbolName::new("unknown".as_bytes()))
        );
    };

    unsafe {
        // mach_task_self(): This function returns the task port
        // for the current process.
        // Essentially, it provides a reference
        // to the current process's task.
        let task_pid = task_for_pid(mach_task_self(), *pid, &mut task);

        if task_pid != kernel_success {
            logs::error_log_with_code(
                "Error attaching the process to task_for_pid. Code:".to_string(),
                task_pid.to_string(),
            );
            exit(1);
        }

        let task_thread = libc::task_threads(task, &mut thread_list, &mut thread_count);

        if task_thread != kernel_success {
            logs::error_log_with_code(
                "Error getting the thread for the given pid. Code:".to_string(),
                task_pid.to_string(),
            );
            exit(1);
        }

        let thread_info = libc::thread_info(
            *thread_list,
            thread_basic_info,
            thread_info_out.as_mut_ptr(),
            &mut thread_info_out_cnt,
        );

        if thread_info != kernel_success {
            logs::error_log_with_code(
                "Error getting info from mach for the given thread. Code:".to_string(),
                task_pid.to_string(),
            );
            exit(1);
        }

        let thread_state = mach2::thread_act::thread_get_state(
            *thread_list,
            6,
            new_state.as_mut_ptr() as *mut _,
            &mut new_state_count,
        );

        if thread_state != kernel_success {
            panic!("Thread_State error: {}", thread_state);
        }
        // addr pointer
        let mut FP = new_state[29];
        // link register (return addr)
        let LR = new_state[30];
        // stack pointer
        let SP = new_state[31];
        // program pointer
        let PC = new_state[32];

        if FP == 0 {
            panic!("FP is 0 cannot unreferenced");
        }

        let fp_ptr = FP as *const u64;

        let pid_i32 = *pid as i32;
        if !fp_ptr.is_null() {
            loop {
                let read_process_address = read_process_address(
                    pid_i32,
                    FP as usize,
                    size,
                    &mut process_address_bytes_buffer,
                )
                .unwrap();

                // load the specific chunk of the buffer into vector
                // used to get the next_fp and next_lr addresses
                next_fp_bytes_vec.copy_from_slice(&process_address_bytes_buffer[..8]);
                next_lr_bytes_vec.copy_from_slice(&process_address_bytes_buffer[8..16]);
                let next_fp = u64::from_le_bytes(next_fp_bytes_vec);
                let next_lr = u64::from_le_bytes(next_lr_bytes_vec);
                addresses.push(next_lr);
                let current_fp = FP;
                FP = next_fp;
                // println!("Next FP: {:#x}, Next LR: {:#x}", current_fp, next_lr);
                if next_fp == 0 {
                    break;
                }
            }
        }
        #[allow(unused_assignments)]
        let mut read_loaded_addr: usize = 0;
        (bin_loaded_addr, read_loaded_addr) = get_binary_based_addr(task);
        parser::parse_bin_file(*pid, addresses, bin_loaded_addr, read_loaded_addr);
    }
    //data output
    println!("binary loaded at: {:#x}", bin_loaded_addr);
    println!("number of threads: {}", thread_count);
    println!(
        "user run time: {}.{:06}ms",
        thread_info_out[0], thread_info_out[1]
    );
    println!(
        "system time: {}.{:06}ms",
        thread_info_out[2], thread_info_out[3]
    );
    println!("cpu usage {}%", thread_info_out[4] as f64 / 10.0);
    // println!("backtrace info {:?}", buf);
}

/// The function `read_process_address` in Rust takes a process ID, address, size, and buffer pointer, reads memory from
/// the specified address in the process, and copies it into the buffer.
///
/// Arguments:
///
/// * `pid`: The `pid` parameter in the function `read_process_address` is of type `Pid`, which likely represents a
/// process identifier.
/// * `address`: The `address` parameter in the `read_process_address` function represents the memory address from which
/// data needs to be read. It is of type `usize`, which is an unsigned integer type representing memory
/// addresses in Rust.
/// * `size`: The `size` parameter in the `read_process_address` function represents the number of bytes to copy from
/// the memory address specified by `address`. It indicates the size of the memory block that needs to
/// be read from the process identified by the `pid`.
/// * `buff`: The `buff` parameter in the `read_process_address` function is a mutable pointer to an array of 128
/// unsigned 8-bit integers (`[u8; 128]`). This pointer is used to store the bytes copied from a
/// specific memory address within a process identified by the `pid` parameter.
///
/// Returns:
///
/// The function `read_process_address` is returning a `std::io::Result<()>`.
fn read_process_address(
    pid: Pid,
    address: usize,
    size: usize,
    buff: *mut [u8; 128],
) -> std::io::Result<()> {
    let handle: ProcessHandle = pid.try_into()?;
    let bytes = copy_address(address, size, &handle)?;
    for i in &bytes {
        if let Some(buff_ref) = unsafe { buff.as_mut() } {
            *buff_ref = bytes.clone().try_into().unwrap();
        }
    }
    Ok(())
}

/// The function `get_binary_based_addr` retrieves the base address of a binary in a target task using
/// low-level Rust code.
///
/// Arguments:
///
/// * `target_task`: The `target_task` parameter is the task for which you want to retrieve the
/// binary-based address. This task is typically a process or application running on the system. The
/// function `get_binary_based_addr` uses this parameter to query the virtual memory region information
/// of the specified task and extract the base address
///
/// Returns:
///
/// The function `get_binary_based_addr` returns the memory address of the specified target task in
/// binary format as a `u64` value.
fn get_binary_based_addr(target_task: u32) -> (u64, usize) {
    // get base addr
    let mut address: mach2::vm_types::mach_vm_address_t = 0;
    let mut size: mach2::vm_types::mach_vm_size_t = 0;
    let flavor: i32 = 9;
    let mut info: mach2::vm_region::vm_region_basic_info_64 = unsafe { std::mem::zeroed() };
    let mut infoCnt: mach2::message::mach_msg_type_number_t =
        std::mem::size_of::<mach2::vm_region::vm_region_basic_info_64>() as u32;
    let mut object_name: mach2::port::mach_port_t = 0;

    // read base addr content
    let mut data: usize = 0;
    let mut dataCnt: mach2::message::mach_msg_type_number_t = 0;

    unsafe {
        let base_addr = mach2::vm::mach_vm_region(
            target_task,
            &mut address,
            &mut size,
            flavor,
            &mut info as *mut _ as *mut _,
            &mut infoCnt,
            &mut object_name,
        );
    }
    unsafe {
        let read_addr =
            mach2::vm::mach_vm_read(target_task, address, size, &mut data, &mut dataCnt);
    }

    return (address, data);
}

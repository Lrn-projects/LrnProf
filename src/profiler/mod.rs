#![allow(unused_variables)]

use crate::logs;
use libc::backtrace;
use libc::exit;
use mach2::traps::mach_task_self;
use mach2::traps::task_for_pid;
embed_plist::embed_info_plist!("../../Info.plist");

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

    // flavor
    let thread_basic_info: u32 = 3;
    let thread_id_info: u32 = 4;
    let thread_extended_info: u32 = 5;
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

        println!(
            "memory where the threads are written: {:?} number of threads of the given process: {}",
            thread_list, thread_count
        );

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

        let mut buf: [*mut libc::c_void; 1024] = [std::ptr::null_mut(); 1024];
        let sz: libc::c_int = 1024;

        let backtrace = backtrace(buf.as_mut_ptr(), sz);

        // println!("backtrace info {:?}", buf);

        // println!("{:?}", thread_info_out);
        println!(
            "user run time: {}.{:06}ms",
            thread_info_out[0], thread_info_out[1]
        );
        println!(
            "system time: {}.{:06}ms",
            thread_info_out[2], thread_info_out[3]
        );

        println!("cpu usage {}%", thread_info_out[4] as f64 / 10.0);

        // let address: u64 = 0; // Define the address
        // let size: u64 = 0; // Define the size
        // let data: *mut usize = std::ptr::null_mut(); // Define the data pointer
        // let data_cnt: *mut u32 = std::ptr::null_mut(); // Define the data count
        // let vm_read = mach2::vm::mach_vm_read(task, address, size, data, data_cnt);
        // println!("{:?}", data);
    }
}

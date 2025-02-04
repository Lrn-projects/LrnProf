#![allow(unused_variables)]
use crate::logs;
use mach2::mach_init::mach_thread_self;
use mach2::traps::mach_task_self;
use mach2::traps::task_for_pid;
embed_plist::embed_info_plist!("../../Info.plist");

pub fn run_profiler(pid: &i32) {
    logs::rp_log("Start running the profiler...");
    let mut task: u32 = 0;
    let kernel_success: i32 = 0;
    //TODO
    // fix segfault by expanding the unsafe block into smaller unsafe block
    //Segfault caused by the unsafe block to huge and to much pointer or reference access inside ?
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
        }
        let mut thread_list: *mut u32 = std::ptr::null_mut();
        let mut thread_count: u32 = 0;
        let task_thread = libc::task_threads(task, &mut thread_list, &mut thread_count);
        println!(
            "memory where the threads are written: {:?} number of threads of the given process: {}",
            thread_list, thread_count
        );

        if task_thread != kernel_success {
            logs::error_log_with_code(
                "Error getting the thread for the given pid. Code:".to_string(),
                task_pid.to_string(),
            );
        }

        // Thread flavor
        let thread_basic_info: u32 = 3;
        let thread_id_info: u32 = 4;
        let thread_sched_info: u32 = 5;
        // let thread_debug_info: u32 = 7;

        let mut thread_info_out: [i32; 1024] = [0; 1024]; // Define the thread_info_out array
        let mut thread_info_out_cnt: u32 = 1024; // Define the thread_info_out_cnt
        let thread_info = libc::thread_info(
            *thread_list,
            thread_basic_info,
            thread_info_out.as_mut_ptr(),
            &mut thread_info_out_cnt,
        );

        if thread_info != kernel_success {
            logs::error_log_with_code(
                "Error getting the thread for the given pid. Code:".to_string(),
                task_pid.to_string(),
            );
        }

        for line in thread_info_out {
            if line != 0 {
                println!("{}", line);
            }
        }

        // let address: u64 = 0; // Define the address
        // let size: u64 = 0; // Define the size
        // let data: *mut usize = std::ptr::null_mut(); // Define the data pointer
        // let data_cnt: *mut u32 = std::ptr::null_mut(); // Define the data count
        // let vm_read = mach2::vm::mach_vm_read(task, address, size, data, data_cnt);
        // println!("{:?}", data);
    }
}

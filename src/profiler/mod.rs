#![allow(unused_variables)]
#![allow(non_snake_case)]

use core::panic;
use std::ptr;

use crate::logs;
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
    // backtrace

    // flavor
    let thread_basic_info: u32 = 3;
    let thread_id_info: u32 = 4;
    let thread_extended_info: u32 = 5;

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

        let mut new_state: [u64; 129] = [0; 129];
        let mut new_state_count: u32 = 129;
        let thread_state = mach2::thread_act::thread_get_state(
            *thread_list,
            6,
            new_state.as_mut_ptr() as *mut _,
            &mut new_state_count,
        );

        if thread_state != kernel_success {
            panic!("Thread_State error: {}", thread_state);
        }
        // frame pointer
        let FP = new_state[29];
        // link register (return addr)
        let LR = new_state[30];
        // stack pointer
        let SP = new_state[31];
        // program pointer
        let PC = new_state[32];

        println!("{} {} {} {}", FP, LR, SP, PC);

        //unwind loop
        let mut addresses: Vec<u64> = Vec::new();

        if FP == 0 {
            panic!("FP is 0 cannot unreferenced");
        }
        // convert fp as raw ptr
        let mut fp = FP as *const u64;

        if !fp.is_null() {
            for i in 0..1 {
                println!("debug: FP {:?}", fp);
                let next_fp = fp as u64;
                println!("debug: next_fp {:?}", next_fp);
                let next_lr = (fp as u64) + 8;
                addresses.push(next_lr);
                println!("Next FP: {:#x}, Next LR: {:#x}", next_fp, next_lr);
                // fp = *(next_fp as *const u64) as *const u64;
            }
        }
        let symbols = backtrace::resolve(FP as *mut libc::c_void, cb);
        println!("symbols: {:?}", symbols);

        for addr in addresses {
            let symbols = backtrace::resolve(FP as *mut libc::c_void, cb);
            println!("symbols: {:?}", symbols);
        }
    }
    //data output
    println!("threads written: {:?}", thread_list);
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

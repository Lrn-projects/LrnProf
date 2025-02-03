use crate::logs;
use mach2::traps::mach_task_self;
use mach2::traps::task_for_pid;
embed_plist::embed_info_plist!("../../Info.plist");

pub fn run_profiler(binary: &i32) {
    logs::rp_log("Start running the profiler...");
    println!("{}", binary);
    let mut task: u32 = 0;
    let kernel_success: i32 = 0;
    unsafe {
        let task_pid = task_for_pid(mach_task_self(), *binary, &mut task);
        if task_pid != kernel_success {
            logs::error_log_with_code(
                "Error attaching the process to task_for_pid. Code:".to_string(),
                task_pid.to_string(),
            );
        }
        let mut thread_list: *mut u32 = std::ptr::null_mut();
        let mut thread_count: u32 = 0;
        libc::task_threads(task, &mut thread_list, &mut thread_count);
        println!("{:?}", thread_list);
    }
}

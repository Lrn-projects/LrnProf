use crate::logs;

pub fn run_profiler(binary: &str) {
    logs::rp_log("Start running the profiler...");
    println!("{}", binary);
}

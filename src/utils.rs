use std::process::{Command, exit};

pub fn command_usage(usage: &str) {
    println!("{}", usage);
    exit(0);
}

pub fn lrnprof_usage() {
    let usage = r"
lrnprof's cli.


Usage: lrnprof command [options]


Commands:
    run             Run the profiler process
    help            Show this help message

Options:

    -h, --help      Show command usage
    -v, --version   Show the current version of RustySpider
";

    println!("{}", usage);
}

pub fn get_bin_path(pid: i32) -> String {
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "lsof -p {} | awk '$4 == \"txt\" {{ print $9 }}'",
            pid
        ))
        .output()
        .expect("Failed to execute command");
    let output_str = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = output_str.split('\n').collect();
    parts[0].to_string()
}

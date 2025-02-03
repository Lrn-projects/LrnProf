use std::process::exit;

pub fn command_usage(usage: &str) {
    println!("{}", usage);
    exit(0);
}

pub fn rustprof_usage() {
    let usage = r"
RustProf's cli.


Usage: RustProf command [options]


Commands:
    run             Run the profiler process
    help            Show this help message

Options:

    -h, --help      Show command usage
    -v, --version   Show the current version of RustySpider
";

    println!("{}", usage);
}

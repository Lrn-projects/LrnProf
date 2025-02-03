pub mod logs;
mod profiler;
pub mod utils;

use std::{env, process::exit};

// Current version of RustProf
// if modified and then running update command it will replace
// your current RustProf installation with the newer version
const VERSION: &'static str = "0.1.0";

#[derive(Debug, Clone)]
enum Commands {
    Run { pid: i32 },
    Version,
    Help,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if let Some(arg) = args.iter().last() {
        match arg.as_str().trim() {
            "-v" => {
                utils::command_usage(&rustprof_version());
            }
            "--version" => {
                utils::command_usage(&rustprof_version());
            }
            _ => {}
        }
    }

    let command = match args.get(1).map(|s| s.as_str()) {
        Some("run") => Commands::Run {
            pid: args
                .get(2)
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or_else(|| {
                    eprintln!("Please provide a valid PID.");
                    exit(1);
                }),
        },
        Some("version") => Commands::Version,
        Some("help") => Commands::Help,
        _ => {
            usage_and_exit("Invalid command".to_string());
            return;
        }
    };

    match command {
        Commands::Run { pid } => profiler::run_profiler(&pid),
        Commands::Version => utils::command_usage(&rustprof_version()),
        Commands::Help => utils::rustprof_usage(),
    }
}

fn usage_and_exit(msg: String) {
    if msg != "" {
        eprintln!("{}", msg);
    }

    utils::rustprof_usage();

    exit(0);
}

pub fn rustprof_version() -> String {
    let usage = format!("rustprof {VERSION}");
    usage
}

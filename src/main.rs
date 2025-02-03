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
    Run { bin: String },
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
            bin: args.get(2).cloned().unwrap_or_else(|| "".to_string()),
        },
        Some("version") => Commands::Version,
        Some("help") => Commands::Help,
        _ => {
            usage_and_exit("Invalid command".to_string());
            return;
        }
    };

    match command {
        Commands::Run { bin } => profiler::run_profiler(&bin),
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
    let usage = format!("nyx {VERSION}");
    usage
}

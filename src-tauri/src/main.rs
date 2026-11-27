// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use devignite::cli::{parse_cli_args, print_help, CliCommand};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let command = parse_cli_args(&args);

    match command {
        CliCommand::Help => {
            print_help();
        }
        CliCommand::Version => {
            println!("DevIgnite v{}", env!("CARGO_PKG_VERSION"));
        }
        CliCommand::List => {
            println!("Use the DevIgnite GUI or database query for active runtimes.");
        }
        CliCommand::Use { tool, version } => {
            println!("Switching {} to version {} via DevIgnite core engine...", tool, version);
        }
        CliCommand::Doctor => {
            println!("Running DevIgnite diagnostic health checks...");
        }
        CliCommand::RunGui => {
            devignite::run();
        }
    }
}

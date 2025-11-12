//! Command-line interface for lex
//! This binary is used to view / convert / process lex files into (and, in the future, from) different formats.
//!
//! Usage:
//!   lex execute --config `<config>` `<path>` [--format `<format>`]  - Execute a pipeline configuration
//!   lex view `<path>`                                            - Open an interactive TUI viewer
//!   lex list-configs                                           - List all available configurations
mod viewer;

use clap::{Arg, Command};
use std::path::PathBuf;

fn main() {
    let matches = Command::new("lex")
        .version(env!("CARGO_PKG_VERSION"))
        .about("A tool for inspecting and processing lex files")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("view")
                .about("Open an interactive TUI viewer")
                .arg(
                    Arg::new("path")
                        .help("Path to the lex file to view")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            Command::new("execute")
                .about("Execute a processing configuration")
                .arg(
                    Arg::new("path")
                        .help("Path to the lex file")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("format")
                        .long("format")
                        .short('f')
                        .help("Output format (e.g., 'ast-tag', 'ast-treeviz')")
                        .default_value("ast-tag"),
                ),
        )
        .subcommand(Command::new("list-configs").about("List available processing configurations"))
        .get_matches();

    // Handle subcommands
    match matches.subcommand() {
        Some(("view", view_matches)) => {
            let path = view_matches.get_one::<String>("path").unwrap();
            handle_view_command(path);
        }
        Some(("execute", execute_matches)) => {
            let path = execute_matches.get_one::<String>("path").unwrap();
            let format = execute_matches.get_one::<String>("format").unwrap();
            handle_execute_command(path, format);
        }
        Some(("list-configs", _)) => {
            handle_list_configs_command();
        }
        _ => unreachable!(),
    }
}

/// Handle the view command
fn handle_view_command(path: &str) {
    let file_path = PathBuf::from(path);
    match viewer::viewer::run_viewer(file_path) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

use lex::lex::pipeline::PipelineExecutor;
/// Handle the execute command
fn handle_execute_command(path: &str, format: &str) {
    let source = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Error reading file: {}", e);
        std::process::exit(1);
    });

    let executor = PipelineExecutor::new();
    let output = executor
        .execute_and_serialize(&source, format)
        .unwrap_or_else(|e| {
            eprintln!("Execution error: {}", e);
            std::process::exit(1);
        });

    print!("{}", output);
}

/// Handle the list-configs command
fn handle_list_configs_command() {
    println!("Available processing configurations:\n");
    println!("  default");
    println!("    The default pipeline.");
}

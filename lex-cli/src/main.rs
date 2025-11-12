//! Command-line interface for lex
//! This binary is used to view / convert / process lex files into (and, in the future, from) different formats.
//!
//! Usage:
//!   lex `<path>` [--format `<format>`]                     - Execute the default pipeline
//!   lex --list-configs                                      - List available processing configurations

use clap::{Arg, ArgAction, Command};
use lex_parser::lex::pipeline::PipelineExecutor;
use std::fs;

fn main() {
    let matches = Command::new("lex")
        .version(env!("CARGO_PKG_VERSION"))
        .about("A tool for inspecting and processing lex files")
        .arg_required_else_help(true)
        .arg(
            Arg::new("path")
                .help("Path to the lex file")
                .required_unless_present("list-configs")
                .index(1),
        )
        .arg(
            Arg::new("format")
                .long("format")
                .short('f')
                .help("Output format (default: ast-tag)")
                .default_value("ast-tag"),
        )
        .arg(
            Arg::new("list-configs")
                .long("list-configs")
                .help("List available processing configurations")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    if matches.get_flag("list-configs") {
        handle_list_configs_command();
        return;
    }

    let path = matches
        .get_one::<String>("path")
        .expect("path is required unless listing configs");
    let format = matches.get_one::<String>("format").unwrap();
    handle_execute_command(path, format);
}

/// Handle the execute command
fn handle_execute_command(path: &str, format: &str) {
    let source = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Error reading file '{}': {}", path, e);
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
    println!("    The simplified default pipeline");
}

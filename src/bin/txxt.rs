//! Command-line interface for txxt-nano
//! This binary is used to view / convert / process txxt files into (and, in the future, from) different formats.
//!
//! Usage:
//!   txxt process `<path>` `<format>`     - Process a file and output to stdout (explicit)
//!   txxt `<path>` `<format>`             - Same as process (default command)
//!   txxt view `<path>`                 - Open an interactive TUI viewer
//!   txxt formats                     - List all available formats
mod viewer;

use clap::{Arg, Command};
use std::collections::HashMap;
use std::path::PathBuf;
use txxt_nano::txxt_nano::processor::{available_formats, ProcessingError};

fn main() {
    // Parse arguments manually to handle --extras-* arguments that clap doesn't know about
    let args: Vec<String> = std::env::args().collect();

    // Filter out --extras-* arguments before passing to clap
    let clap_args: Vec<&str> = args
        .iter()
        .filter(|arg| {
            // Keep all args except those that start with --extras-
            !arg.starts_with("--extras-")
        })
        .map(|arg| arg.as_str())
        .collect();

    let matches = Command::new("txxt")
        .version(env!("CARGO_PKG_VERSION"))
        .about("A tool for inspecting and processing txxt files")
        .subcommand_required(false)
        .arg_required_else_help(true)
        // Default command args (for backwards compatibility)
        .arg(
            Arg::new("path")
                .help("Path to the txxt file to process")
                .index(1),
        )
        .arg(
            Arg::new("format")
                .help("Output format (e.g., token-simple, token-json)")
                .index(2),
        )
        // Subcommands
        .subcommand(
            Command::new("process")
                .about("Process a file and output to stdout (default command)")
                .arg(
                    Arg::new("path")
                        .help("Path to the txxt file to process")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("format")
                        .help("Output format (e.g., token-simple, token-json)")
                        .required(true)
                        .index(2),
                ),
        )
        .subcommand(
            Command::new("view")
                .about("Open an interactive TUI viewer")
                .arg(
                    Arg::new("path")
                        .help("Path to the txxt file to view")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(Command::new("formats").about("List all available output formats"))
        .try_get_matches_from(clap_args)
        .unwrap_or_else(|e| {
            eprintln!("{}", e);
            std::process::exit(1);
        });

    // Handle subcommands or default command
    match matches.subcommand() {
        Some(("process", process_matches)) => {
            let path = process_matches.get_one::<String>("path").unwrap();
            let format_str = process_matches.get_one::<String>("format").unwrap();
            handle_process_command(path, format_str);
        }
        Some(("view", view_matches)) => {
            let path = view_matches.get_one::<String>("path").unwrap();
            handle_view_command(path);
        }
        Some(("formats", _)) => {
            handle_formats_command();
        }
        None => {
            // Default command: treat as process
            let path = matches.get_one::<String>("path");
            let format = matches.get_one::<String>("format");

            match (path, format) {
                (Some(p), Some(f)) => handle_process_command(p, f),
                _ => {
                    // This shouldn't happen because arg_required_else_help(true) will show help
                    // if required args are missing. But just in case:
                    std::process::exit(1);
                }
            }
        }
        _ => unreachable!(),
    }
}

/// Handle the process command
fn handle_process_command(path: &str, format_str: &str) {
    // Parse extras from raw arguments (everything after format that starts with --extras-)
    let extras = parse_extras_from_args();

    match process_file_with_format(path, format_str, extras) {
        Ok(output) => print!("{}", output),
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!("\nAvailable formats:");
            for format in available_formats() {
                eprintln!("  {}", format);
            }
            std::process::exit(1);
        }
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

/// Handle the formats command
fn handle_formats_command() {
    println!("Available formats:");
    for format in available_formats() {
        println!("  {}", format);
    }
}

/// Parse extras from raw command line arguments
/// Expects arguments in the format `--extras-<key>` `<value>`
fn parse_extras_from_args() -> HashMap<String, String> {
    let mut result = HashMap::new();

    let args: Vec<String> = std::env::args().collect();

    // Look for arguments that start with --extras-
    for arg in args.iter().skip(3) {
        // Skip path and format arguments (first 3 args: executable, path, format)
        if arg.starts_with("--extras-") {
            if let Some(rest) = arg.strip_prefix("--extras-") {
                if let Some((key, val)) = rest.split_once('=') {
                    result.insert(key.to_string(), val.to_string());
                }
            }
        }
    }

    result
}

/// Process a file with the given format string and extras
fn process_file_with_format(
    path: &str,
    format_str: &str,
    extras: HashMap<String, String>,
) -> Result<String, ProcessingError> {
    let spec = txxt_nano::txxt_nano::processor::ProcessingSpec::from_string(format_str)?;
    txxt_nano::txxt_nano::processor::process_file_with_extras(path, &spec, extras)
}

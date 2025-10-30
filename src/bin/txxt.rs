//! Command-line interface for txxt-nano

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
        .about("A tool for inspecting txxt files")
        .arg(
            Arg::new("path")
                .help("Path to the txxt file to process")
                .required_unless_present("list-formats")
                .index(1),
        )
        .arg(
            Arg::new("format")
                .help("Output format (e.g., token-simple, token-json, app)")
                .required_unless_present("list-formats")
                .index(2),
        )
        .arg(
            Arg::new("list-formats")
                .long("list-formats")
                .help("List all available formats")
                .action(clap::ArgAction::SetTrue),
        )
        .try_get_matches_from(clap_args)
        .unwrap_or_else(|e| {
            eprintln!("{}", e);
            std::process::exit(1);
        });

    if matches.get_flag("list-formats") {
        println!("Available formats:");
        for format in available_formats() {
            println!("  {}", format);
        }
        return;
    }

    let path = matches.get_one::<String>("path").unwrap();
    let format_str = matches.get_one::<String>("format").unwrap();

    // Handle "app" format - launch the viewer
    if format_str == "app" {
        let file_path = PathBuf::from(path);
        match viewer::viewer_main::run_viewer(file_path) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    // Parse extras from raw arguments (everything after format that starts with --extras-)
    let extras = parse_extras_from_args();

    match process_file_with_format(path, format_str, extras) {
        Ok(output) => print!("{}", output),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

/// Parse extras from raw command line arguments
/// Expects arguments in the format --extras-<key>=<value>
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

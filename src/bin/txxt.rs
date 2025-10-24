//! Command-line interface for txxt-nano

use clap::{Arg, Command};
use txxt_nano::txxt_nano::processor::{available_formats, ProcessingError};

fn main() {
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
                .help("Output format (e.g., token-simple, token-json)")
                .required_unless_present("list-formats")
                .index(2),
        )
        .arg(
            Arg::new("list-formats")
                .long("list-formats")
                .help("List all available formats")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    if matches.get_flag("list-formats") {
        println!("Available formats:");
        for format in available_formats() {
            println!("  {}", format);
        }
        return;
    }

    let path = matches.get_one::<String>("path").unwrap();
    let format_str = matches.get_one::<String>("format").unwrap();

    match process_file_with_format(path, format_str) {
        Ok(output) => print!("{}", output),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

/// Process a file with the given format string
fn process_file_with_format(path: &str, format_str: &str) -> Result<String, ProcessingError> {
    let spec = txxt_nano::txxt_nano::processor::ProcessingSpec::from_string(format_str)?;
    txxt_nano::txxt_nano::processor::process_file(path, &spec)
}

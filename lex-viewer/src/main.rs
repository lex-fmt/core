//! Standalone binary for the lex interactive viewer.
//! Usage:
//!   lexv <path>

mod viewer;

use clap::{Arg, Command, ValueHint};
use std::path::PathBuf;

fn main() {
    let matches = Command::new("lexv")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Interactive terminal viewer for lex documents")
        .arg(
            Arg::new("path")
                .help("Path to the lex document to open")
                .required(true)
                .index(1)
                .value_hint(ValueHint::FilePath),
        )
        .get_matches();

    let path = matches.get_one::<String>("path").unwrap();
    if let Err(err) = viewer::viewer::run_viewer(PathBuf::from(path)) {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

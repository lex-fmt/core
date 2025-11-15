//! Command-line interface for lex
//!
//! This binary is used to view / convert / process lex files into different formats.
//!
//! Usage:
//!   lex <path> <transform>     - Execute a transform (e.g., "ast-tag", "token-core-json")
//!   lex --list-transforms       - List available transforms

mod transforms;

use clap::{Arg, ArgAction, Command};
use std::fs;

fn main() {
    let matches = Command::new("lex")
        .version(env!("CARGO_PKG_VERSION"))
        .about("A tool for inspecting and processing lex files")
        .arg_required_else_help(true)
        .arg(
            Arg::new("path")
                .help("Path to the lex file")
                .required_unless_present("list-transforms")
                .index(1),
        )
        .arg(
            Arg::new("transform")
                .help("Transform to apply (stage-format, e.g., 'ast-tag', 'token-core-json')")
                .required_unless_present("list-transforms")
                .value_parser(clap::builder::PossibleValuesParser::new(
                    transforms::AVAILABLE_TRANSFORMS,
                ))
                .index(2),
        )
        .arg(
            Arg::new("list-transforms")
                .long("list-transforms")
                .help("List available transforms")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    if matches.get_flag("list-transforms") {
        handle_list_transforms_command();
        return;
    }

    let path = matches
        .get_one::<String>("path")
        .expect("path is required unless listing transforms");
    let transform = matches
        .get_one::<String>("transform")
        .expect("transform is required unless listing transforms");

    handle_execute_command(path, transform);
}

/// Handle the execute command
fn handle_execute_command(path: &str, transform: &str) {
    let source = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Error reading file '{}': {}", path, e);
        std::process::exit(1);
    });

    let output = transforms::execute_transform(&source, transform).unwrap_or_else(|e| {
        eprintln!("Execution error: {}", e);
        std::process::exit(1);
    });

    print!("{}", output);
}

/// Handle the list-transforms command
fn handle_list_transforms_command() {
    println!("Available transforms:\n");
    println!("Stages:");
    println!("  token-core  - Core tokenization (no semantic indentation)");
    println!("  token-line  - Full lexing with semantic indentation");
    println!("  ir          - Intermediate representation (parse tree)");
    println!("  ast         - Abstract syntax tree (final parsed document)\n");

    println!("Formats:");
    println!("  json        - JSON output (all stages)");
    println!("  tag         - XML-like tag format (AST only)");
    println!("  treeviz     - Tree visualization (AST only)");
    println!("  simple      - Plain text token names\n");

    println!("Available transform combinations:");
    for transform_name in transforms::AVAILABLE_TRANSFORMS {
        println!("  {}", transform_name);
    }
}

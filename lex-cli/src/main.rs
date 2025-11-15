// Command-line interface for lex
//
// This binary provides commands for inspecting and converting lex files.
//
// The inspect command is an internal tool for aid in the development of the lex ecosystem, and is bound to be be extracted to it's own crate in the future.
//
// The main role for the lex program is to interface with lex content. Be it converting to and fro, linting or formatting it.
// The core capabilities use the lex-babel crate. This crate being a interface for the lex-babel library, which is a collection of formats and transformers.
//
// Usage:
//  lex <input> --to <format> [--from <format>] [--output <file>]  - Convert between formats (default)
//  lex convert <input> --from <format> --to <format> [--output <file>]  - Same as above (explicit)
//  lex inspect <path> <transform>        - Execute a transform (e.g., "ast-tag", "token-core-json")
//  lex --list-transforms                 - List available transforms

mod transforms;

use clap::{Arg, ArgAction, Command, ValueHint};
use lex_babel::FormatRegistry;
use std::fs;

fn main() {
    let matches = Command::new("lex")
        .version(env!("CARGO_PKG_VERSION"))
        .about("A tool for inspecting and converting lex files")
        .arg_required_else_help(true)
        // Root-level convert arguments (for default command)
        .arg(
            Arg::new("input")
                .help("Input file path")
                .index(1)
                .value_hint(ValueHint::FilePath)
                .conflicts_with("list-transforms"),
        )
        .arg(
            Arg::new("from")
                .long("from")
                .help("Source format (auto-detected from file extension if not specified)")
                .value_hint(ValueHint::Other)
                .requires("input"),
        )
        .arg(
            Arg::new("to")
                .long("to")
                .help("Target format")
                .value_hint(ValueHint::Other)
                .requires("input"),
        )
        .arg(
            Arg::new("output")
                .long("output")
                .short('o')
                .help("Output file path (defaults to stdout)")
                .value_hint(ValueHint::FilePath)
                .requires("input"),
        )
        .arg(
            Arg::new("list-transforms")
                .long("list-transforms")
                .help("List available transforms")
                .action(ArgAction::SetTrue)
                .global(true),
        )
        .subcommand(
            Command::new("inspect")
                .about("Inspect internal representations of lex files")
                .arg(
                    Arg::new("path")
                        .help("Path to the lex file")
                        .required(true)
                        .index(1)
                        .value_hint(ValueHint::FilePath),
                )
                .arg(
                    Arg::new("transform")
                        .help(
                            "Transform to apply (stage-format, e.g., 'ast-tag', 'token-core-json')",
                        )
                        .required(true)
                        .value_parser(clap::builder::PossibleValuesParser::new(
                            transforms::AVAILABLE_TRANSFORMS,
                        ))
                        .index(2)
                        .value_hint(ValueHint::Other),
                ),
        )
        .subcommand(
            Command::new("convert")
                .about("Convert between document formats (default command)")
                .arg(
                    Arg::new("input")
                        .help("Input file path")
                        .required(true)
                        .index(1)
                        .value_hint(ValueHint::FilePath),
                )
                .arg(
                    Arg::new("from")
                        .long("from")
                        .help("Source format")
                        .required(true)
                        .value_hint(ValueHint::Other),
                )
                .arg(
                    Arg::new("to")
                        .long("to")
                        .help("Target format")
                        .required(true)
                        .value_hint(ValueHint::Other),
                )
                .arg(
                    Arg::new("output")
                        .long("output")
                        .short('o')
                        .help("Output file path (defaults to stdout)")
                        .value_hint(ValueHint::FilePath),
                ),
        )
        .get_matches();

    if matches.get_flag("list-transforms") {
        handle_list_transforms_command();
        return;
    }

    match matches.subcommand() {
        Some(("inspect", sub_matches)) => {
            let path = sub_matches
                .get_one::<String>("path")
                .expect("path is required");
            let transform = sub_matches
                .get_one::<String>("transform")
                .expect("transform is required");
            handle_inspect_command(path, transform);
        }
        Some(("convert", sub_matches)) => {
            let input = sub_matches
                .get_one::<String>("input")
                .expect("input is required");
            let from = sub_matches
                .get_one::<String>("from")
                .expect("from is required");
            let to = sub_matches.get_one::<String>("to").expect("to is required");
            let output = sub_matches.get_one::<String>("output").map(|s| s.as_str());
            handle_convert_command(input, from, to, output);
        }
        None => {
            // No subcommand - treat as default convert command
            if let Some(input) = matches.get_one::<String>("input") {
                let from_arg = matches.get_one::<String>("from");
                let to = matches.get_one::<String>("to");

                // Auto-detect --from if not provided
                let from = if let Some(f) = from_arg {
                    f.to_string()
                } else {
                    let registry = FormatRegistry::default();
                    match registry.detect_format_from_filename(input) {
                        Some(detected) => detected,
                        None => {
                            eprintln!("Error: Could not detect format from filename '{}'", input);
                            eprintln!("Please specify --from explicitly");
                            std::process::exit(1);
                        }
                    }
                };

                if to.is_none() {
                    eprintln!("Error: --to is required for conversion");
                    eprintln!(
                        "Usage: lex <input> [--from <format>] --to <format> [--output <file>]"
                    );
                    std::process::exit(1);
                }

                let output = matches.get_one::<String>("output").map(|s| s.as_str());
                handle_convert_command(input, &from, to.unwrap(), output);
            } else {
                eprintln!("Error: No command specified. Use --help for usage information.");
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("Unknown subcommand. Use --help for usage information.");
            std::process::exit(1);
        }
    }
}

/// Handle the inspect command (old execute command)
fn handle_inspect_command(path: &str, transform: &str) {
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

/// Handle the convert command
fn handle_convert_command(input: &str, from: &str, to: &str, output: Option<&str>) {
    let registry = FormatRegistry::default();

    // Validate formats exist
    if let Err(e) = registry.get(from) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
    if let Err(e) = registry.get(to) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    // Read input file
    let source = fs::read_to_string(input).unwrap_or_else(|e| {
        eprintln!("Error reading file '{}': {}", input, e);
        std::process::exit(1);
    });

    // Parse
    let doc = registry.parse(&source, from).unwrap_or_else(|e| {
        eprintln!("Parse error: {}", e);
        std::process::exit(1);
    });

    // Serialize
    let result = registry.serialize(&doc, to).unwrap_or_else(|e| {
        eprintln!("Serialization error: {}", e);
        std::process::exit(1);
    });

    // Output
    match output {
        Some(path) => {
            fs::write(path, &result).unwrap_or_else(|e| {
                eprintln!("Error writing file '{}': {}", path, e);
                std::process::exit(1);
            });
        }
        None => {
            print!("{}", result);
        }
    }
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
    println!("  simple      - Plain text token names");
    println!("  pprint      - Pretty-printed token names\n");

    println!("Available transform combinations:");
    for transform_name in transforms::AVAILABLE_TRANSFORMS {
        println!("  {}", transform_name);
    }

    println!("\nConversion formats:");
    let registry = FormatRegistry::default();
    for format_name in registry.list_formats() {
        println!("  {}", format_name);
    }
}

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
                    Arg::new("config")
                        .long("config")
                        .short('c')
                        .help("Configuration name (e.g., 'default', 'linebased', 'tokens-indentation')")
                        .required(true),
                )
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
                        .help("Output format (default: ast-tag for Document, token-json for Tokens)")
                        .default_value("auto"),
                ),
        )
        .subcommand(
            Command::new("list-configs").about("List available processing configurations"),
        )
        .get_matches();

    // Handle subcommands
    match matches.subcommand() {
        Some(("view", view_matches)) => {
            let path = view_matches.get_one::<String>("path").unwrap();
            handle_view_command(path);
        }
        Some(("execute", execute_matches)) => {
            let config = execute_matches.get_one::<String>("config").unwrap();
            let path = execute_matches.get_one::<String>("path").unwrap();
            let format = execute_matches.get_one::<String>("format").unwrap();
            handle_execute_command(config, path, format);
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

/// Handle the execute command
fn handle_execute_command(config: &str, path: &str, format: &str) {
    use lex::lex::pipeline::{DocumentLoader, ExecutionOutput};

    let loader = DocumentLoader::new();
    let output = loader.load_and_execute(path, config).unwrap_or_else(|e| {
        eprintln!("Execution error: {}", e);
        eprintln!("\nAvailable configurations:");
        for config in loader.executor().list_configs() {
            eprintln!("  {} - {}", config.name, config.description);
        }
        std::process::exit(1);
    });

    // Format and print output
    let formatted = match (output, format) {
        // Serialized output is already formatted, use directly
        (ExecutionOutput::Serialized(s), _) => s,
        (ExecutionOutput::Document(doc), "auto") | (ExecutionOutput::Document(doc), "ast-tag") => {
            lex::lex::parsing::serialize_ast_tag(&doc)
        }
        (ExecutionOutput::Document(doc), "ast-treeviz") => lex::lex::parsing::to_treeviz_str(&doc),
        (ExecutionOutput::Tokens(stream), "auto")
        | (ExecutionOutput::Tokens(stream), "token-json") => {
            let tokens = stream.unroll();
            serde_json::to_string_pretty(&tokens).unwrap_or_else(|e| {
                eprintln!("Error formatting tokens: {}", e);
                std::process::exit(1);
            })
        }
        (ExecutionOutput::Tokens(stream), "token-simple") => {
            let tokens = stream.unroll();
            tokens
                .iter()
                .map(|(token, _)| format!("{}", token))
                .collect::<Vec<_>>()
                .join("")
        }
        (ExecutionOutput::Document(_), fmt) => {
            eprintln!("Format '{}' not supported for Document output", fmt);
            eprintln!("Available formats for Document: ast-tag, ast-treeviz");
            std::process::exit(1);
        }
        (ExecutionOutput::Tokens(_), fmt) => {
            eprintln!("Format '{}' not supported for Tokens output", fmt);
            eprintln!("Available formats for Tokens: token-json, token-simple");
            std::process::exit(1);
        }
    };

    print!("{}", formatted);
}

/// Handle the list-configs command
fn handle_list_configs_command() {
    use lex::lex::pipeline::DocumentLoader;

    let loader = DocumentLoader::new();
    println!("Available processing configurations:\n");

    for config in loader.executor().list_configs() {
        println!("  {}", config.name);
        println!("    {}", config.description);
        println!();
    }
}

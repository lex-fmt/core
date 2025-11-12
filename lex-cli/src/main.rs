//! Command-line interface for lex
//! This binary is used to view / convert / process lex files into (and, in the future, from) different formats.
//!
//! Usage:
//!   lex `<path>` --config `<config>` [--format `<format>`]   - Execute a pipeline configuration
//!   lex --list-configs                                      - List all available configurations

use clap::{Arg, ArgAction, Command};

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
            Arg::new("config")
                .long("config")
                .short('c')
                .help("Configuration name (e.g., 'default', 'linebased', 'tokens-indentation')")
                .required_unless_present("list-configs"),
        )
        .arg(
            Arg::new("format")
                .long("format")
                .short('f')
                .help("Output format (default: ast-tag for Document, token-json for Tokens)")
                .default_value("auto"),
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

    let config = matches
        .get_one::<String>("config")
        .expect("config is required unless listing configs");
    let path = matches
        .get_one::<String>("path")
        .expect("path is required unless listing configs");
    let format = matches.get_one::<String>("format").unwrap();
    handle_execute_command(config, path, format);
}

/// Handle the execute command
fn handle_execute_command(config: &str, path: &str, format: &str) {
    use lex_parser::lex::pipeline::{DocumentLoader, ExecutionOutput};

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
            lex_parser::lex::parsing::serialize_ast_tag(&doc)
        }
        (ExecutionOutput::Document(doc), "ast-treeviz") => {
            lex_parser::lex::parsing::to_treeviz_str(&doc)
        }
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
    use lex_parser::lex::pipeline::DocumentLoader;

    let loader = DocumentLoader::new();
    println!("Available processing configurations:\n");

    for config in loader.executor().list_configs() {
        println!("  {}", config.name);
        println!("    {}", config.description);
        println!();
    }
}

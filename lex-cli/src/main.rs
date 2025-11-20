// Command-line interface for lex
//
// This binary provides commands for inspecting and converting lex files.
//
// The inspect command is an internal tool for aid in the development of the lex ecosystem, and is bound to be be extracted to it's own crate in the future.
//
// The main role for the lex program is to interface with lex content. Be it converting to and fro, linting or formatting it.
// The core capabilities use the lex-babel crate. This crate being a interface for the lex-babel library, which is a collection of formats and transformers.
//
// Converting:
//
// The conversion needs a to and from pair. The to can be auto-detected from the file extension, while being overwrittable by an explicit --from flag.
// Usage:
//  lex <input> --to <format> [--from <format>] [--output <file>]  - Convert between formats (default)
//  lex convert <input> --to <format> [--from <format>] [--output <file>]  - Same as above (explicit)
//  lex inspect <path> [<transform>]      - Execute a transform (defaults to "ast-treeviz")
//  lex --list-transforms                 - List available transforms
//
// Extra Parameters:
//
// Format-specific parameters can be passed using --extra-<parameter-name> <value>.
// The CLI layer strips the "extra-" prefix and passes the parameters to the format/transform.
// Example:
//  lex inspect file.lex --extra-all-nodes true --extra-max-depth 5

mod transforms;

use clap::{Arg, ArgAction, Command, ValueHint};
use lex_babel::FormatRegistry;
use std::collections::HashMap;
use std::fs;

/// Parse extra-* arguments from command line args
/// Returns (cleaned_args_without_extras, extra_params_map)
///
/// Supports both:
/// - `--extra-<key> <value>` (explicit value)
/// - `--extra-<key>` (boolean flag, defaults to "true")
fn parse_extra_args(args: &[String]) -> (Vec<String>, HashMap<String, String>) {
    let mut cleaned_args = Vec::new();
    let mut extra_params = HashMap::new();
    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];

        if let Some(key) = arg.strip_prefix("--extra-") {
            // Found an extra-* argument
            // Check if the next arg is a value or another flag/end
            let has_value = if i + 1 < args.len() {
                let next = &args[i + 1];
                !next.starts_with('-') && !next.starts_with("--")
            } else {
                false
            };

            if has_value {
                // Explicit value provided
                extra_params.insert(key.to_string(), args[i + 1].clone());
                i += 2; // Skip both the key and value
            } else {
                // No value, treat as boolean flag (default to "true")
                extra_params.insert(key.to_string(), "true".to_string());
                i += 1;
            }
            continue;
        }

        cleaned_args.push(arg.clone());
        i += 1;
    }

    (cleaned_args, extra_params)
}

fn build_cli() -> Command {
    Command::new("lex")
        .version(env!("CARGO_PKG_VERSION"))
        .about("A tool for inspecting and converting lex files")
        .arg_required_else_help(true)
        .subcommand_required(false)
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
                            "Transform to apply (stage-format, e.g., 'ast-tag', 'token-core-json'). Defaults to 'ast-treeviz'",
                        )
                        .required(false)
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
                        .help("Source format (auto-detected from file extension if not specified)")
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
}

fn main() {
    // Try to parse args. If no subcommand is provided, inject "convert"
    let args: Vec<String> = std::env::args().collect();

    // Parse extra-* arguments before clap processing
    let (cleaned_args, extra_params) = parse_extra_args(&args);

    // First, try normal parsing with cleaned args
    let cli = build_cli();
    let matches = match cli.clone().try_get_matches_from(&cleaned_args) {
        Ok(m) => m,
        Err(e) => {
            // Check if this is a "missing subcommand" error by seeing if the first arg looks like a file
            if cleaned_args.len() > 1
                && !cleaned_args[1].starts_with('-')
                && cleaned_args[1] != "inspect"
                && cleaned_args[1] != "convert"
                && cleaned_args[1] != "help"
            {
                // Inject "convert" as the subcommand
                let mut new_args = vec![cleaned_args[0].clone(), "convert".to_string()];
                new_args.extend_from_slice(&cleaned_args[1..]);

                // Try parsing again with "convert" injected
                match cli.try_get_matches_from(&new_args) {
                    Ok(m) => m,
                    Err(e2) => e2.exit(),
                }
            } else {
                // Not a case where we should inject convert, show original error
                e.exit();
            }
        }
    };

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
                .map(|s| s.as_str())
                .unwrap_or("ast-treeviz");
            handle_inspect_command(path, transform, &extra_params);
        }
        Some(("convert", sub_matches)) => {
            let input = sub_matches
                .get_one::<String>("input")
                .expect("input is required");
            let from_arg = sub_matches.get_one::<String>("from");
            let to = sub_matches.get_one::<String>("to").expect("to is required");

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

            let output = sub_matches.get_one::<String>("output").map(|s| s.as_str());
            handle_convert_command(input, &from, to, output, &extra_params);
        }
        _ => {
            eprintln!("Unknown subcommand. Use --help for usage information.");
            std::process::exit(1);
        }
    }
}

/// Handle the inspect command (old execute command)
fn handle_inspect_command(path: &str, transform: &str, extra_params: &HashMap<String, String>) {
    let source = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Error reading file '{}': {}", path, e);
        std::process::exit(1);
    });

    let output =
        transforms::execute_transform(&source, transform, extra_params).unwrap_or_else(|e| {
            eprintln!("Execution error: {}", e);
            std::process::exit(1);
        });

    print!("{}", output);
}

/// Handle the convert command
fn handle_convert_command(
    input: &str,
    from: &str,
    to: &str,
    output: Option<&str>,
    extra_params: &HashMap<String, String>,
) {
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
    // TODO: Pass extra_params to serialize when the API supports it
    if !extra_params.is_empty() {
        eprintln!("Warning: extra parameters are not yet supported for convert command");
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_extra_args_empty() {
        let args = vec![
            "lex".to_string(),
            "inspect".to_string(),
            "file.lex".to_string(),
        ];
        let (cleaned, extra) = parse_extra_args(&args);

        assert_eq!(cleaned, args);
        assert!(extra.is_empty());
    }

    #[test]
    fn test_parse_extra_args_single_param() {
        let args = vec![
            "lex".to_string(),
            "inspect".to_string(),
            "file.lex".to_string(),
            "--extra-all-nodes".to_string(),
            "true".to_string(),
        ];
        let (cleaned, extra) = parse_extra_args(&args);

        assert_eq!(
            cleaned,
            vec![
                "lex".to_string(),
                "inspect".to_string(),
                "file.lex".to_string()
            ]
        );
        assert_eq!(extra.len(), 1);
        assert_eq!(extra.get("all-nodes"), Some(&"true".to_string()));
    }

    #[test]
    fn test_parse_extra_args_multiple_params() {
        let args = vec![
            "lex".to_string(),
            "inspect".to_string(),
            "file.lex".to_string(),
            "--extra-all-nodes".to_string(),
            "true".to_string(),
            "ast-treeviz".to_string(),
            "--extra-max-depth".to_string(),
            "5".to_string(),
        ];
        let (cleaned, extra) = parse_extra_args(&args);

        assert_eq!(
            cleaned,
            vec![
                "lex".to_string(),
                "inspect".to_string(),
                "file.lex".to_string(),
                "ast-treeviz".to_string()
            ]
        );
        assert_eq!(extra.len(), 2);
        assert_eq!(extra.get("all-nodes"), Some(&"true".to_string()));
        assert_eq!(extra.get("max-depth"), Some(&"5".to_string()));
    }

    #[test]
    fn test_parse_extra_args_mixed_with_regular_args() {
        let args = vec![
            "lex".to_string(),
            "convert".to_string(),
            "input.lex".to_string(),
            "--to".to_string(),
            "html".to_string(),
            "--extra-theme".to_string(),
            "dark".to_string(),
            "--from".to_string(),
            "lex".to_string(),
        ];
        let (cleaned, extra) = parse_extra_args(&args);

        assert_eq!(
            cleaned,
            vec![
                "lex".to_string(),
                "convert".to_string(),
                "input.lex".to_string(),
                "--to".to_string(),
                "html".to_string(),
                "--from".to_string(),
                "lex".to_string()
            ]
        );
        assert_eq!(extra.len(), 1);
        assert_eq!(extra.get("theme"), Some(&"dark".to_string()));
    }

    #[test]
    fn test_parse_extra_args_boolean_flag() {
        let args = vec![
            "lex".to_string(),
            "inspect".to_string(),
            "file.lex".to_string(),
            "ast-tag".to_string(),
            "--extra-ast-full".to_string(),
        ];
        let (cleaned, extra) = parse_extra_args(&args);

        assert_eq!(
            cleaned,
            vec![
                "lex".to_string(),
                "inspect".to_string(),
                "file.lex".to_string(),
                "ast-tag".to_string()
            ]
        );
        assert_eq!(extra.len(), 1);
        assert_eq!(extra.get("ast-full"), Some(&"true".to_string()));
    }

    #[test]
    fn test_parse_extra_args_boolean_flag_at_end() {
        let args = vec![
            "lex".to_string(),
            "inspect".to_string(),
            "file.lex".to_string(),
            "--extra-verbose".to_string(),
        ];
        let (cleaned, extra) = parse_extra_args(&args);

        assert_eq!(
            cleaned,
            vec![
                "lex".to_string(),
                "inspect".to_string(),
                "file.lex".to_string()
            ]
        );
        assert_eq!(extra.len(), 1);
        assert_eq!(extra.get("verbose"), Some(&"true".to_string()));
    }

    #[test]
    fn test_parse_extra_args_mixed_boolean_and_value() {
        let args = vec![
            "lex".to_string(),
            "inspect".to_string(),
            "file.lex".to_string(),
            "--extra-verbose".to_string(),
            "--extra-max-depth".to_string(),
            "5".to_string(),
            "--extra-compact".to_string(),
        ];
        let (cleaned, extra) = parse_extra_args(&args);

        assert_eq!(
            cleaned,
            vec![
                "lex".to_string(),
                "inspect".to_string(),
                "file.lex".to_string()
            ]
        );
        assert_eq!(extra.len(), 3);
        assert_eq!(extra.get("verbose"), Some(&"true".to_string()));
        assert_eq!(extra.get("max-depth"), Some(&"5".to_string()));
        assert_eq!(extra.get("compact"), Some(&"true".to_string()));
    }
}

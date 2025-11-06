//! Example usage of the lex_sources library
//!
//! This demonstrates how to use the lex_sources library to access
//! verified lex sample files for testing.

use lex::lex::processor::lex_sources::LexSources;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Lex Sources Library Demo ===\n");

    // List all available samples
    println!("Available samples:");
    for sample in LexSources::list_samples() {
        println!("  - {}", sample);
    }
    println!();

    // Get raw string content
    println!("=== Raw String Content ===");
    let content = LexSources::get_string("000-paragraphs.lex")?;
    println!("First 100 characters of 000-paragraphs.lex:");
    println!("{}", &content[..content.len().min(100)]);
    println!();

    // Get tokenized content
    println!("=== Tokenized Content ===");
    let tokens_json = LexSources::get_tokens("040-lists.lex")?;
    println!("First 200 characters of tokenized 040-lists.lex:");
    println!("{}", &tokens_json[..tokens_json.len().min(200)]);
    println!();

    // Get processed content
    println!("=== Processed Content ===");
    let processed = LexSources::get_processed("050-paragraph-lists.lex", "token-simple")?;
    println!("First 200 characters of processed 050-paragraph-lists.lex:");
    println!("{}", &processed[..processed.len().min(200)]);
    println!();

    // Get sample metadata
    println!("=== Sample Metadata ===");
    let info = LexSources::get_sample_info("000-paragraphs.lex")?;
    println!("Sample: {}", info.filename);
    println!("Spec Version: {}", info.spec_version);
    println!("Lines: {}", info.line_count);
    println!("Characters: {}", info.char_count);
    println!("Description: {:?}", info.description);

    Ok(())
}

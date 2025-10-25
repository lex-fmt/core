//! Example usage of the txxt_sources library
//!
//! This demonstrates how to use the txxt_sources library to access
//! verified txxt sample files for testing.

use txxt_nano::txxt_nano::processor::txxt_sources::TxxtSources;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Txxt Sources Library Demo ===\n");

    // List all available samples
    println!("Available samples:");
    for sample in TxxtSources::list_samples() {
        println!("  - {}", sample);
    }
    println!();

    // Get raw string content
    println!("=== Raw String Content ===");
    let content = TxxtSources::get_string("000-paragraphs.txxt")?;
    println!("First 100 characters of 000-paragraphs.txxt:");
    println!("{}", &content[..content.len().min(100)]);
    println!();

    // Get tokenized content
    println!("=== Tokenized Content ===");
    let tokens_json = TxxtSources::get_tokens("040-lists.txxt")?;
    println!("First 200 characters of tokenized 040-lists.txxt:");
    println!("{}", &tokens_json[..tokens_json.len().min(200)]);
    println!();

    // Get processed content
    println!("=== Processed Content ===");
    let processed = TxxtSources::get_processed("050-paragraph-lists.txxt", "token-simple")?;
    println!("First 200 characters of processed 050-paragraph-lists.txxt:");
    println!("{}", &processed[..processed.len().min(200)]);
    println!();

    // Get sample metadata
    println!("=== Sample Metadata ===");
    let info = TxxtSources::get_sample_info("000-paragraphs.txxt")?;
    println!("Sample: {}", info.filename);
    println!("Spec Version: {}", info.spec_version);
    println!("Lines: {}", info.line_count);
    println!("Characters: {}", info.char_count);
    println!("Description: {:?}", info.description);

    Ok(())
}

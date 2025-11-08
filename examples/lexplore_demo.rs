//! Demonstration of the Lexplore test harness
//!
//! This example shows how to use Lexplore to access verified lex sample files
//! for testing and development purposes.

use lex::lex::testing::lexplore::{DocumentType, ElementType, Lexplore};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Lexplore Demo ===\n");

    // 1. Loading individual elements
    println!("=== Loading Individual Elements ===");
    let paragraph_source = Lexplore::get_source_for(ElementType::Paragraph, 1)?;
    println!(
        "Paragraph #1 (first 100 chars): {}",
        &paragraph_source.chars().take(100).collect::<String>()
    );

    let list_source = Lexplore::get_source_for(ElementType::List, 1)?;
    println!(
        "List #1 (first 100 chars): {}",
        &list_source.chars().take(100).collect::<String>()
    );

    // 2. Loading full documents
    println!("\n=== Loading Full Documents ===");
    let trifecta_doc = Lexplore::get_document_source_for(DocumentType::Trifecta, 0)?;
    println!(
        "Trifecta #0 (first 100 chars): {}",
        &trifecta_doc.chars().take(100).collect::<String>()
    );

    // 3. Using the fluent API to parse
    println!("\n=== Using Fluent API ===");
    let doc = Lexplore::paragraph(1).parse();
    let paragraph = doc.expect_paragraph();
    println!("Parsed paragraph has {} lines", paragraph.lines.len());

    // 4. List available element numbers
    println!("\n=== Available Elements ===");
    let paragraph_numbers = Lexplore::list_numbers_for(ElementType::Paragraph)?;
    println!("Available paragraph variations: {:?}", paragraph_numbers);

    let list_numbers = Lexplore::list_numbers_for(ElementType::List)?;
    println!("Available list variations: {:?}", list_numbers);

    println!("\n=== Demo Complete ===");
    Ok(())
}

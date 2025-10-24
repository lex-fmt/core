//! Command-line interface for txxt-nano

use txxt_nano::txxt_nano::Parser;

fn main() {
    println!("txxt-nano CLI tool");
    println!("Version: {}", env!("CARGO_PKG_VERSION"));

    let _parser = Parser::new();
    println!("Parser initialized successfully");
}

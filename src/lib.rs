//! # txxt
//!
//! A parser for the txxt format.
//!
//! ## Testing
//!
//! For comprehensive testing guidelines, see the [testing module](txxt::testing).
//! All parser tests must follow strict rules using verified txxt sources and AST assertions.

pub mod txxt;

/// A simple function to demonstrate the library works
pub fn hello() -> &'static str {
    "Hello from txxt!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello() {
        assert_eq!(hello(), "Hello from txxt!");
    }
}

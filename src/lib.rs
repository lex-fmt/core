//! # txxt-nano
//!
//! A parser for the txxt format.

pub mod txxt_nano;

/// A simple function to demonstrate the library works
pub fn hello() -> &'static str {
    "Hello from txxt-nano!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello() {
        assert_eq!(hello(), "Hello from txxt-nano!");
    }
}

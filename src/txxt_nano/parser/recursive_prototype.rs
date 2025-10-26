//! Prototype for understanding Recursive::declare() pattern
//!
//! This module tests mutual recursion between two simple parsers to validate
//! the pattern before applying it to the full document parser.

use chumsky::prelude::*;

#[derive(Debug, Clone, PartialEq)]
enum Item {
    A(Box<Item>),
    B(Box<Item>),
    Leaf(char),
}

/// Test mutual recursion: A can contain B, B can contain A
/// Grammar:
///   A → 'a' | 'a' B
///   B → 'b' | 'b' A
#[allow(dead_code)]
fn test_mutual_recursion() -> impl Parser<char, Item, Error = Simple<char>> {
    // STEP 1: Declare both recursive parsers
    let mut a_parser = Recursive::declare();
    let mut b_parser = Recursive::declare();

    // STEP 2: Define what each parser does, referencing the other
    a_parser.define(
        just('a')
            .then(b_parser.clone().or_not())
            .map(|(c, b_opt)| match b_opt {
                Some(b) => Item::A(Box::new(b)),
                None => Item::Leaf(c),
            }),
    );

    b_parser.define(
        just('b')
            .then(a_parser.clone().or_not())
            .map(|(c, a_opt)| match a_opt {
                Some(a) => Item::B(Box::new(a)),
                None => Item::Leaf(c),
            }),
    );

    // STEP 3: Return a choice of both
    a_parser.or(b_parser)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_a() {
        let parser = test_mutual_recursion();
        let result = parser.parse("a").unwrap();
        assert_eq!(result, Item::Leaf('a'));
    }

    #[test]
    fn test_simple_b() {
        let parser = test_mutual_recursion();
        let result = parser.parse("b").unwrap();
        assert_eq!(result, Item::Leaf('b'));
    }

    #[test]
    fn test_a_then_b() {
        let parser = test_mutual_recursion();
        let result = parser.parse("ab").unwrap();
        assert_eq!(result, Item::A(Box::new(Item::Leaf('b'))));
    }

    #[test]
    fn test_b_then_a() {
        let parser = test_mutual_recursion();
        let result = parser.parse("ba").unwrap();
        assert_eq!(result, Item::B(Box::new(Item::Leaf('a'))));
    }

    #[test]
    fn test_nested_aba() {
        let parser = test_mutual_recursion();
        let result = parser.parse("aba").unwrap();
        assert_eq!(
            result,
            Item::A(Box::new(Item::B(Box::new(Item::Leaf('a')))))
        );
    }
}

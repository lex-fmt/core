//! Test factories for creating locations and spanned tokens succinctly

use std::ops::Range;

use crate::txxt::lexer::Token;

/// Canonical alias for spanned tokens used across tests
pub type Tokens = Vec<(Token, Range<usize>)>;

/// Make a byte range location
pub fn make_loc(start: usize, end: usize) -> Range<usize> {
    start..end
}

/// Make a single spanned token
pub fn mk_token(token: Token, start: usize, end: usize) -> (Token, Range<usize>) {
    (token, make_loc(start, end))
}

/// Make a vector of spanned tokens from a list of (Token, start, end)
pub fn mk_tokens(specs: &[(Token, usize, usize)]) -> Tokens {
    specs
        .iter()
        .cloned()
        .map(|(t, s, e)| mk_token(t, s, e))
        .collect()
}

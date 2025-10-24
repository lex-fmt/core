# Parser Architecture

## Tokenizer with Log

```rust
#[derive(Logos, Debug, PartialEq)]
enum Token {
    #[token("if")]
    If,
    #[regex("[a-zA-Z]+")]
    Ident,
    
    // Don't skip newlines - track them
    #[token("\n")]
    Newline,
    
    Indent,  // Generated manually
    Dedent,  // Generated manually
}
```

## Indentation Wrapper

Logos doesn't emit INDENT/DEDENT directly. Write a wrapper iterator:

```rust
struct IndentLexer<'source> {
    lexer: Lexer<'source, Token>,
    indent_stack: Vec<usize>,
    pending: VecDeque<Token>,
}

impl Iterator for IndentLexer<'_> {
    fn next(&mut self) -> Option<Token> {
        // If we have pending INDENT/DEDENT, return those first
        if let Some(tok) = self.pending.pop_front() {
            return Some(tok);
        }
        
        match self.lexer.next()? {
            Token::Newline => {
                // Count whitespace after newline
                let indent = count_indent(&self.lexer);
                let current = *self.indent_stack.last().unwrap();
                
                if indent > current {
                    self.indent_stack.push(indent);
                    self.pending.push_back(Token::Indent);
                } else if indent < current {
                    // Pop stack and emit Dedents
                    while indent < *self.indent_stack.last().unwrap() {
                        self.indent_stack.pop();
                        self.pending.push_back(Token::Dedent);
                    }
                }
                self.next() // Skip the newline itself
            }
            tok => Some(tok)
        }
    }
}
```

## Parse with Chumsky

```rust
fn parser() -> impl Parser<Token, Ast, Error = Simple<Token>> {
    let block = expr
        .repeated()
        .delimited_by(just(Token::Indent), just(Token::Dedent));
    
    // INDENT/DEDENT work like { }
    let if_stmt = just(Token::If)
        .then(expr)
        .then(block)
        .map(|((_, cond), body)| Ast::If(cond, body));
    
    // ...
}
```

**Key insight:** Normalize indentation to delimiter tokens in the wrapper, then chumsky just parses them as regular block markers. Keeps the parser context-free.

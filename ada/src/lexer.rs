use std::{iter::Peekable, str::CharIndices};

use crate::token::{Token, TokenKind};

pub struct Lexer<'a> {
    /// Iterator over all chars in the file
    chars: Peekable<CharIndices<'a>>,

    /// The original source text
    source: &'a str,
}

impl<'a> Lexer<'a> {
    /// Get all tokens for a source file
    pub fn run(source: &'a str) -> Vec<Token> {
        let mut lexer = Lexer {
            chars: source.char_indices().peekable(),
            source,
        };

        let mut tokens = vec![];
        loop {
            let tok = lexer.next();
            if tok.kind == TokenKind::Eof {
                break;
            }
            tokens.push(tok);
        }
        tokens
    }

    /// Get the next available token
    fn next(&mut self) -> Token {
        todo!()
    }
}

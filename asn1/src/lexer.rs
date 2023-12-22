use std::str::CharIndices;

use crate::{
    token::{Token, TokenKind},
    util::{Peek, Peekable},
};

/// State for converting a source string into a token stream
#[derive(Debug, Clone)]
pub struct Lexer<'a> {
    /// Iterator over all chars in the file
    chars: Peekable<CharIndices<'a>>,

    /// The original source text
    source: &'a str,

    /// File ID to use for all returned tokens
    file: usize,
}

impl<'a> Lexer<'a> {
    /// Create a new Lexer for a given source file.  `file` represents a file
    /// ID that will be returned with each token.
    pub fn new(file: usize, source: &'a str) -> Self {
        Self {
            chars: source.char_indices().n_peekable(),
            source,
            file,
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(&(_, c)) = self.chars.peek(0) {
            if !is_whitespace(c) {
                break;
            }
            self.chars.next();
        }

        let (offset, c) = self.chars.next()?;

        match c {
            '{' => self.simple_token(TokenKind::LeftCurly, offset),
            '}' => self.simple_token(TokenKind::RightCurly, offset),
            '<' => self.simple_token(TokenKind::Less, offset),
            '>' => self.simple_token(TokenKind::Greater, offset),
            ',' => self.simple_token(TokenKind::Comma, offset),
            '.' => self.simple_token(TokenKind::Dot, offset),
            '/' => self.simple_token(TokenKind::ForwardSlash, offset),
            '(' => self.simple_token(TokenKind::LeftParen, offset),
            ')' => self.simple_token(TokenKind::RightParen, offset),
            '[' => self.simple_token(TokenKind::LeftSquare, offset),
            ']' => self.simple_token(TokenKind::RightSquare, offset),
            '-' => self.simple_token(TokenKind::Hyphen, offset),
            ':' => self.simple_token(TokenKind::Colon, offset),
            '=' => self.simple_token(TokenKind::Equals, offset),
            '"' => self.simple_token(TokenKind::DoubleQuote, offset),
            '\'' => self.simple_token(TokenKind::SingleQuote, offset),
            ';' => self.simple_token(TokenKind::SemiColon, offset),
            '@' => self.simple_token(TokenKind::At, offset),
            '|' => self.simple_token(TokenKind::Pipe, offset),
            '!' => self.simple_token(TokenKind::Exclamation, offset),
            '^' => self.simple_token(TokenKind::Caret, offset),

            _ => self.simple_token(TokenKind::Error, offset),
        }
    }
}

impl<'a> Lexer<'a> {
    /// Return a 1 character token
    fn simple_token(&self, kind: TokenKind, offset: usize) -> Option<Token<'a>> {
        let value = &self.source[offset..];
        // Get first character of value
        let first = value.chars().next()?;
        Some(Token {
            kind,
            value: &value[..first.len_utf8()],
            offset,
            file: self.file,
        })
    }
}

fn is_whitespace(c: char) -> bool {
    // A0 = Non breaking space
    "\t \u{A0}".contains(c) || is_newline(c)
}

fn is_newline(c: char) -> bool {
    // 0B = Vertical Tab
    // 0C = Form Feed
    "\n\x0B\x0C\r".contains(c)
}

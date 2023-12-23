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
            '(' => self.simple_token(TokenKind::LeftParen, offset),
            ')' => self.simple_token(TokenKind::RightParen, offset),
            '[' => self.simple_token(TokenKind::LeftSquare, offset),
            ']' => self.simple_token(TokenKind::RightSquare, offset),
            '=' => self.simple_token(TokenKind::Equals, offset),
            ';' => self.simple_token(TokenKind::SemiColon, offset),
            '@' => self.simple_token(TokenKind::At, offset),
            '|' => self.simple_token(TokenKind::Pipe, offset),
            '!' => self.simple_token(TokenKind::Exclamation, offset),
            '^' => self.simple_token(TokenKind::Caret, offset),

            ':' => self
                .multi_token(TokenKind::Assignment, offset, "::=")
                .or_else(|| self.simple_token(TokenKind::Colon, offset)),

            // ITU-T X.680 (02/2021) 11.8: Hyphen and non-breaking hyphen are
            // to be treated as identical in all names (including reserved words)
            '-' | '\u{2011}' => self
                .single_comment(c, offset)
                .or_else(|| self.simple_token(TokenKind::Hyphen, offset)),

            '/' => self
                .multi_comment(offset)
                .or_else(|| self.simple_token(TokenKind::ForwardSlash, offset)),

            c if c.is_ascii_alphabetic() => self.identifier(offset),

            _ => self.simple_token(TokenKind::Unrecognised, offset),
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

    /// Try to return a multi-character token
    fn multi_token(&mut self, kind: TokenKind, offset: usize, value: &str) -> Option<Token<'a>> {
        let tok_value = &self.source[offset..];

        if !tok_value.starts_with(value) {
            return None;
        }

        for _ in 0..value.len() - 1 {
            self.chars.next();
        }

        Some(Token {
            kind,
            value: &tok_value[..value.len()],
            offset,
            file: self.file,
        })
    }

    /// Parse a single line comment which is text between pairs of two hyphens.
    /// Non-breaking hyphens are also accepted instead of hyphens.
    fn single_comment(&mut self, first: char, offset: usize) -> Option<Token<'a>> {
        let value = &self.source[offset..];

        let &(_, second) = self.chars.peek(0)?;
        if !matches!(second, '-' | '\u{2011}') {
            return None;
        }
        self.chars.next(); // Consume the second hyphen

        let mut len = first.len_utf8() + second.len_utf8();

        while let Some(&(_, next)) = self.chars.peek(0) {
            if is_newline(next) {
                break;
            }

            len += next.len_utf8();

            if matches!(next, '-' | '\u{2011}') {
                if let Some(&(_, c @ ('-' | '\u{2011}'))) = self.chars.peek(1) {
                    len += c.len_utf8();

                    self.chars.next();
                    self.chars.next();
                    break;
                }
            }

            self.chars.next();
        }

        Some(Token {
            kind: TokenKind::SingleComment,
            value: &value[..len],
            offset,
            file: self.file,
        })
    }

    /// Parse a multi line comment which is text between `/*` and `*/`.  The comment
    /// ends when a matching `*/` has been found for every `/*` encountered.
    fn multi_comment(&mut self, offset: usize) -> Option<Token<'a>> {
        let value = &self.source[offset..];

        let &(_, c) = self.chars.peek(0)?;
        if c != '*' {
            // not a start of comment
            return None;
        }
        self.chars.next();

        let mut len = 2;
        let mut depth = 1;
        while let Some(&(_, c)) = self.chars.peek(0) {
            len += c.len_utf8();
            self.chars.next();

            if c == '/' && matches!(self.chars.peek(0), Some((_, '*'))) {
                len += 1;
                depth += 1;
                self.chars.next();
            } else if c == '*' && matches!(self.chars.peek(0), Some((_, '/'))) {
                len += 1;
                depth -= 1;
                self.chars.next();

                if depth == 0 {
                    break;
                }
            }
        }

        let kind = match depth {
            0 => TokenKind::MultiComment,
            _ => TokenKind::NonTerminatedComment,
        };

        Some(Token {
            kind,
            value: &value[..len],
            offset,
            file: self.file,
        })
    }

    /// Parse an identifier.  Could be a type reference, identifier, value reference
    /// or module reference.
    fn identifier(&mut self, offset: usize) -> Option<Token<'a>> {
        let value = &self.source[offset..];

        let mut len = 1;
        while let Some(&(_, c)) = self.chars.peek(0) {
            if c.is_ascii_alphanumeric() || "$_".contains(c) {
                len += 1;
                self.chars.next();
                continue;
            }
            if c == '-' {
                if let Some(&(_, c)) = self.chars.peek(1) {
                    if c.is_ascii_alphanumeric() {
                        len += 2;
                        self.chars.next();
                        self.chars.next();
                        continue;
                    }
                }
            }

            break;
        }

        Some(Token {
            kind: TokenKind::Identifier,
            value: &value[..len],
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

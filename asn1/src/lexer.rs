use std::{
    collections::{HashMap, VecDeque},
    str::CharIndices,
    sync::OnceLock,
};

use crate::{
    token::{self, Token, TokenKind},
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

    /// List of comment tokens not returned yet
    comments: VecDeque<Token<'a>>,

    /// A cached identifier lexed at a given offset.
    identifier: Option<(usize, Token<'a>)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LexerError {
    /// Unable to lex one of the token kinds at a given offset into a file
    Expected {
        kind: &'static [TokenKind],
        offset: usize,
        file: usize,
    },

    UnexpectedCharacter {
        ch: char,
        file: usize,
        offset: usize,
    },

    EndOfFile {
        file: usize,
    },

    NonTerminatedComment {
        offset: usize,
        file: usize,
    },

    ParserDepthExceeded,
}

pub type Result<T = (), E = LexerError> = std::result::Result<T, E>;

impl<'a> Lexer<'a> {
    /// Create a new Lexer for a given source file.  `file` represents a file
    /// ID that will be returned with each token.
    pub fn new(file: usize, source: &'a str) -> Self {
        Self {
            chars: source.char_indices().n_peekable(),
            source,
            file,
            comments: VecDeque::new(),
            identifier: None,
        }
    }

    /// Return the next token from the source. If there are no more characters
    /// in the source text, returns an error.  Skips all whitespace and comments
    /// before the start of a token.
    pub fn peek(&mut self) -> Result<Token<'a>> {
        // skip whitespace and comments
        while let Some(&(offset, c)) = self.chars.peek(0) {
            match c {
                '-' | '\u{2011}' => {
                    if !self.single_comment(c, offset) {
                        break;
                    }
                }
                '/' => {
                    if !self.multi_comment(offset)? {
                        break;
                    }
                }
                _ if is_whitespace(c) => {
                    self.chars.next();
                }
                _ => break,
            }
        }

        let &(offset, c) = self
            .chars
            .peek(0)
            .ok_or(LexerError::EndOfFile { file: self.file })?;

        let ret = match c {
            '{' => self.simple_token(TokenKind::LeftCurly, offset),
            '}' => self.simple_token(TokenKind::RightCurly, offset),
            '>' => self.simple_token(TokenKind::Greater, offset),
            ',' => self.simple_token(TokenKind::Comma, offset),
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
            '_' => self.simple_token(TokenKind::Underscore, offset),
            '-' | '\u{2011}' => self.simple_token(TokenKind::Hyphen, offset),

            ':' => self.multi_token(TokenKind::Colon, TokenKind::Assignment, offset, "::="),

            '<' => self.multi_token(TokenKind::Less, TokenKind::XMLEndTag, offset, "</"),

            '/' => self.multi_token(
                TokenKind::ForwardSlash,
                TokenKind::XMLSingleTagEnd,
                offset,
                "/>",
            ),

            '.' => self.multi_token(TokenKind::Dot, TokenKind::Ellipsis, offset, "..."),

            'a'..='z' | 'A'..='Z' => self.identifier(c, offset),

            '"' => self.c_string(offset),

            '0'..='9' => self.number(offset),

            ch => {
                return Err(LexerError::UnexpectedCharacter {
                    ch,
                    file: self.file,
                    offset,
                })
            }
        };

        Ok(ret)
    }

    /// Peeks a token of the given kind (see peek()), then advances the source
    /// text past the token.  Might also return a comment token instead.
    pub fn next_token(&mut self) -> Result<Token<'a>> {
        let peek = self.peek();

        if let Some(comment) = self.comments.pop_front() {
            return Ok(comment);
        }

        match peek {
            Ok(t) => {
                for _ in t.value.chars() {
                    self.chars.next();
                }

                Ok(t)
            }
            Err(e) => Err(e),
        }
    }

    /// Returns true if the lexer is at the end of its source file
    pub fn is_eof(&mut self) -> bool {
        matches!(self.peek(), Err(LexerError::EndOfFile { .. }))
    }

    /// Return a 1 character token
    fn simple_token(&self, kind: TokenKind, offset: usize) -> Token<'a> {
        let value = &self.source[offset..];

        Token {
            kind,
            value: &value[..1],
            offset,
            file: self.file,
        }
    }

    /// Try to return a multi-character token
    fn multi_token(
        &mut self,
        single_kind: TokenKind,
        multi_kind: TokenKind,
        offset: usize,
        value: &str,
    ) -> Token<'a> {
        let tok_value = &self.source[offset..];

        if !tok_value.starts_with(value) {
            return Token {
                kind: single_kind,
                value: &tok_value[..1],
                offset,
                file: self.file,
            };
        }

        Token {
            kind: multi_kind,
            value: &tok_value[..value.len()],
            offset,
            file: self.file,
        }
    }

    /// Parse a single line comment which is text between pairs of two hyphens.
    /// Non-breaking hyphens are also accepted instead of hyphens.
    fn single_comment(&mut self, first: char, offset: usize) -> bool {
        let value = &self.source[offset..];

        let Some(&(_, second)) = self.chars.peek(1) else { return false };
        if !matches!(second, '-' | '\u{2011}') {
            return false;
        }
        self.chars.next(); // Consume the first hyphen
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

        self.comments.push_back(Token {
            kind: TokenKind::SingleComment,
            value: &value[..len],
            offset,
            file: self.file,
        });

        true
    }

    /// Parse a multi line comment which is text between `/*` and `*/`.  The comment
    /// ends when a matching `*/` has been found for every `/*` encountered.
    fn multi_comment(&mut self, offset: usize) -> Result<bool> {
        let value = &self.source[offset..];

        let Some(&(_, c)) = self.chars.peek(1) else { return Ok(false)};
        if c != '*' {
            // not a start of comment
            return Ok(false);
        }

        self.chars.next();
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

        if depth != 0 {
            return Err(LexerError::NonTerminatedComment {
                offset,
                file: self.file,
            });
        }

        self.comments.push_back(Token {
            kind: TokenKind::MultiComment,
            value: &value[..len],
            offset,
            file: self.file,
        });

        Ok(true)
    }

    /// Parse an identifier.  Could be a type reference, identifier, value reference
    /// or module reference.  Does not consume the identifier, len characters must
    /// be skipped after the identifier is parsed if the identifier is used.
    fn identifier(&mut self, first: char, offset: usize) -> Token<'a> {
        // cache to speed up matching against a lot of different key words
        if let Some((o, ident)) = self.identifier {
            if o == offset {
                return ident;
            }
        }

        let value = &self.source[offset..];

        let mut len = 1;
        while let Some(&(_, c)) = self.chars.peek(len) {
            if c.is_ascii_alphanumeric() || "$_".contains(c) {
                len += 1;
                continue;
            }

            if c == '-' || c == '\u{2011}' {
                if let Some(&(_, c)) = self.chars.peek(len + 1) {
                    if c.is_ascii_alphanumeric() {
                        // does not check the hyphen as it does not count as
                        // lower or upper case
                        len += 2;
                        continue;
                    }
                }
            }

            break;
        }

        let ident_kind = if first.is_ascii_lowercase() {
            TokenKind::ValueRefOrIdent
        } else {
            TokenKind::TypeOrModuleRef
        };

        let value = &value[..len];
        let kind = keywords().get(value).copied().unwrap_or(ident_kind);

        Token {
            kind,
            value,
            offset,
            file: self.file,
        }
    }

    /// Parse a number ([1-9][0-9]*)|0
    fn number(&mut self, offset: usize) -> Token<'a> {
        let value = &self.source[offset..];
        let mut len = 1;
        while let Some(&(_, ch)) = self.chars.peek(len) {
            if !ch.is_ascii_digit() {
                break;
            }

            len += 1;
        }

        let value = &value[..len];
        // TODO: if value.starts_with('0') && len > 1 {
        //     return None;
        // }

        Token {
            kind: TokenKind::Number,
            value,
            offset,
            file: self.file,
        }
    }

    /// Parse a character string literal
    fn c_string(&mut self, offset: usize) -> Token<'a> {
        let value = &self.source[offset..];
        let mut len = 1;

        while let Some(&(_, ch)) = self.chars.peek(len) {
            len += ch.len_utf8();

            if ch == '"' {
                if matches!(self.chars.peek(len), Some(&(_, '"'))) {
                    len += 1;
                } else {
                    break;
                }
            }
        }

        let value = &value[..len];

        Token {
            kind: TokenKind::CString,
            value,
            offset,
            file: self.file,
        }
    }
}

/// Is the character any valid whitespace
fn is_whitespace(c: char) -> bool {
    // A0 = Non breaking space
    "\t \u{A0}".contains(c) || is_newline(c)
}

/// Is the character a valid newline character
fn is_newline(c: char) -> bool {
    // 0B = Vertical Tab
    // 0C = Form Feed
    "\n\x0B\x0C\r".contains(c)
}

/// Get a mapping from keyword strings to their token kind
fn keywords() -> &'static HashMap<&'static str, TokenKind> {
    static KEYWORDS: OnceLock<HashMap<&'static str, TokenKind>> = OnceLock::new();
    KEYWORDS.get_or_init(|| HashMap::from(token::KEYWORD_DATA))
}

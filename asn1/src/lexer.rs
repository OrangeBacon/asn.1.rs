use std::{
    collections::{HashMap, VecDeque},
    str::CharIndices,
    sync::OnceLock,
};

use crate::{
    compiler::SourceId,
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
    pub(crate) id: SourceId,

    /// List of comment tokens not returned yet
    comments: VecDeque<Token>,

    /// How square brackets are currently being parsed
    square_bracket_mode: SquareBracketMode,

    /// Should keyword parsing be enabled
    enable_keywords: bool,
}

/// How should lexing of square brackets proceed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum SquareBracketMode {
    /// Lex `[[` as two tokens
    Split,

    /// Lex `[[` as one open version bracket
    #[default]
    Join,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LexerError {
    /// Unexpected character within the input file
    UnexpectedCharacter {
        ch: char,
        id: SourceId,
        offset: usize,
    },

    /// End of file reached when trying to get a token
    EndOfFile { id: SourceId },

    /// Reached end of file while parsing a multi-line comment
    NonTerminatedComment { offset: usize, id: SourceId },

    /// Reached end of file while parsing a character string
    NonTerminatedString { offset: usize, id: SourceId },

    /// Reached end of file while parsing a binary or hexadecimal string
    NonTerminatedBHString { offset: usize, id: SourceId },

    /// No identifier found after an `&`
    MissingFieldName { offset: usize, id: SourceId },

    /// Unexpected keyword after an '&'
    KeywordFieldName { offset: usize, id: SourceId },
}

pub type Result<T = (), E = LexerError> = std::result::Result<T, E>;

impl<'a> Lexer<'a> {
    /// Create a new Lexer for a given source file.  `file` represents a file
    /// ID that will be returned with each token.
    pub fn new(id: SourceId, source: &'a str) -> Self {
        Self {
            chars: source.char_indices().n_peekable(),
            source,
            id,
            comments: VecDeque::new(),
            square_bracket_mode: Default::default(),
            enable_keywords: true,
        }
    }

    /// Get the current character offset.  Might be inaccurate due to comment
    /// tokens possibly having been consumed or not.
    pub fn offset(&mut self) -> usize {
        self.chars
            .peek(0)
            .map(|(o, _)| *o)
            .unwrap_or(self.source.len())
    }

    /// Return the next token from the source. If there are no more characters
    /// in the source text, returns an error.  Skips all whitespace and comments
    /// before the start of a token.
    pub fn peek(&mut self) -> Result<Token> {
        self.skip_trivia()?;

        let &(offset, c) = self
            .chars
            .peek(0)
            .ok_or(LexerError::EndOfFile { id: self.id })?;

        let ret = match c {
            '{' => self.simple_token(TokenKind::LeftCurly, offset),
            '}' => self.simple_token(TokenKind::RightCurly, offset),
            '>' => self.simple_token(TokenKind::Greater, offset),
            ',' => self.simple_token(TokenKind::Comma, offset),
            '(' => self.simple_token(TokenKind::LeftParen, offset),
            ')' => self.simple_token(TokenKind::RightParen, offset),
            '=' => self.simple_token(TokenKind::Equals, offset),
            ';' => self.simple_token(TokenKind::SemiColon, offset),
            '@' => self.simple_token(TokenKind::At, offset),
            '|' => self.simple_token(TokenKind::Pipe, offset),
            '!' => self.simple_token(TokenKind::Exclamation, offset),
            '^' => self.simple_token(TokenKind::Caret, offset),
            '_' => self.simple_token(TokenKind::Underscore, offset),
            '<' => self.simple_token(TokenKind::Less, offset),
            '/' => self.simple_token(TokenKind::ForwardSlash, offset),
            '-' | '\u{2011}' => self.simple_token(TokenKind::Hyphen, offset),

            ':' => self.multi_token(TokenKind::Colon, TokenKind::Assignment, offset, "::="),
            '.' => self.multi_token(TokenKind::Dot, TokenKind::Ellipsis, offset, "..."),

            '[' if self.square_bracket_mode == SquareBracketMode::Join => {
                self.multi_token(TokenKind::LeftSquare, TokenKind::VersionOpen, offset, "[[")
            }
            ']' if self.square_bracket_mode == SquareBracketMode::Join => self.multi_token(
                TokenKind::RightSquare,
                TokenKind::VersionClose,
                offset,
                "]]",
            ),
            '[' if self.square_bracket_mode == SquareBracketMode::Split => {
                self.simple_token(TokenKind::LeftSquare, offset)
            }
            ']' if self.square_bracket_mode == SquareBracketMode::Split => {
                self.simple_token(TokenKind::RightSquare, offset)
            }

            '&' => self.field(offset)?,
            'a'..='z' | 'A'..='Z' => self.identifier(c, offset),

            '"' => self.c_string(offset)?,
            '\'' => self.bh_string(offset)?,

            '0'..='9' => self.number(offset),

            ch => {
                return Err(LexerError::UnexpectedCharacter {
                    ch,
                    id: self.id,
                    offset,
                })
            }
        };

        Ok(ret)
    }

    /// Peeks a token of the given kind (see peek()), then advances the source
    /// text past the token.  Might also return a comment token instead.
    pub fn next_token(&mut self) -> Result<Token> {
        let peek = self.peek();

        if let Some(comment) = self.comments.pop_front() {
            return Ok(comment);
        }

        match peek {
            Ok(t) => {
                for _ in 0..t.length {
                    self.chars.next();
                }

                Ok(t)
            }
            Err(e) => Err(e),
        }
    }

    /// Peek the next XML token from the source.
    pub fn peek_xml(&mut self) -> Result<Token> {
        let &(offset, ch) = self
            .chars
            .peek(0)
            .ok_or(LexerError::EndOfFile { id: self.id })?;

        Ok(match ch {
            '<' => {
                if matches!(self.chars.peek(1), Some((_, '/'))) {
                    Token {
                        kind: TokenKind::XMLEndTag,
                        length: 2,
                        offset,
                        id: self.id,
                    }
                } else {
                    self.simple_token(TokenKind::Less, offset)
                }
            }
            '>' => self.simple_token(TokenKind::Greater, offset),
            '/' if matches!(self.chars.peek(1), Some((_, '>'))) => Token {
                kind: TokenKind::XMLSingleTagEnd,
                length: 2,
                offset,
                id: self.id,
            },
            _ => {
                let mut length = ch.len_utf8();
                while let Some(&(_, ch)) = self.chars.peek(length) {
                    match (ch, self.chars.peek(length + ch.len_utf8())) {
                        ('<' | '>', _) | ('/', Some((_, '>'))) => break,
                        _ => (),
                    }
                    length += ch.len_utf8();
                }

                Token {
                    kind: TokenKind::XMLData,
                    length,
                    offset,
                    id: self.id,
                }
            }
        })
    }

    /// Consume the next XML token
    pub fn next_xml(&mut self) -> Result<Token> {
        let peek = self.peek_xml();

        match peek {
            Ok(t) => {
                for _ in 0..t.length {
                    self.chars.next();
                }

                Ok(t)
            }
            Err(e) => Err(e),
        }
    }

    /// Consume the next comment
    pub fn next_comment(&mut self) -> Option<Token> {
        self.comments.pop_front()
    }

    /// Returns true if the lexer is at the end of its source file
    pub fn is_eof(&mut self) -> bool {
        matches!(self.peek(), Err(LexerError::EndOfFile { .. }))
    }

    /// skip whitespace and comments
    fn skip_trivia(&mut self) -> Result {
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

        Ok(())
    }

    /// Return a 1 character token
    fn simple_token(&self, kind: TokenKind, offset: usize) -> Token {
        Token {
            kind,
            length: 1,
            offset,
            id: self.id,
        }
    }

    /// Try to return a multi-character token
    fn multi_token(
        &mut self,
        single_kind: TokenKind,
        multi_kind: TokenKind,
        offset: usize,
        value: &str,
    ) -> Token {
        let tok_value = &self.source[offset..];

        if !tok_value.starts_with(value) {
            return Token {
                kind: single_kind,
                length: 1,
                offset,
                id: self.id,
            };
        }

        Token {
            kind: multi_kind,
            length: value.len(),
            offset,
            id: self.id,
        }
    }

    /// Parse a single line comment which is text between pairs of two hyphens.
    /// Non-breaking hyphens are also accepted instead of hyphens.
    fn single_comment(&mut self, first: char, offset: usize) -> bool {
        let Some(&(_, second)) = self.chars.peek(1) else { return false };
        if !matches!(second, '-' | '\u{2011}') {
            return false;
        }
        self.chars.next(); // Consume the first hyphen
        self.chars.next(); // Consume the second hyphen

        let mut length = first.len_utf8() + second.len_utf8();

        while let Some(&(_, next)) = self.chars.peek(0) {
            if is_newline(next) {
                break;
            }

            length += next.len_utf8();

            if matches!(next, '-' | '\u{2011}') {
                if let Some(&(_, c @ ('-' | '\u{2011}'))) = self.chars.peek(1) {
                    length += c.len_utf8();

                    self.chars.next();
                    self.chars.next();
                    break;
                }
            }

            self.chars.next();
        }

        self.comments.push_back(Token {
            kind: TokenKind::SingleComment,
            length,
            offset,
            id: self.id,
        });

        true
    }

    /// Parse a multi line comment which is text between `/*` and `*/`.  The comment
    /// ends when a matching `*/` has been found for every `/*` encountered.
    fn multi_comment(&mut self, offset: usize) -> Result<bool> {
        let Some(&(_, c)) = self.chars.peek(1) else { return Ok(false)};
        if c != '*' {
            // not a start of comment
            return Ok(false);
        }

        self.chars.next();
        self.chars.next();

        let mut length = 2;
        let mut depth = 1;
        while let Some(&(_, c)) = self.chars.peek(0) {
            length += c.len_utf8();
            self.chars.next();

            if c == '/' && matches!(self.chars.peek(0), Some((_, '*'))) {
                length += 1;
                depth += 1;
                self.chars.next();
            } else if c == '*' && matches!(self.chars.peek(0), Some((_, '/'))) {
                length += 1;
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
                id: self.id,
            });
        }

        self.comments.push_back(Token {
            kind: TokenKind::MultiComment,
            length,
            offset,
            id: self.id,
        });

        Ok(true)
    }

    /// Parse an identifier.  Could be a type reference, identifier, value reference
    /// or module reference.  Does not consume the identifier, len characters must
    /// be skipped after the identifier is parsed if the identifier is used.
    fn identifier(&mut self, first: char, offset: usize) -> Token {
        let value = &self.source[offset..];

        let mut length = 1;
        while let Some(&(_, c)) = self.chars.peek(length) {
            if c.is_ascii_alphanumeric() || "$_".contains(c) {
                length += 1;
                continue;
            }

            if c == '-' || c == '\u{2011}' {
                if let Some(&(_, c)) = self.chars.peek(length + 1) {
                    if c.is_ascii_alphanumeric() {
                        // does not check the hyphen as it does not count as
                        // lower or upper case
                        length += 2;
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

        let value = &value[..length];
        let kind = if self.enable_keywords {
            keywords().get(value).copied().unwrap_or(ident_kind)
        } else {
            ident_kind
        };

        Token {
            kind,
            length,
            offset,
            id: self.id,
        }
    }

    /// Parse a number ([1-9][0-9]*)|0
    fn number(&mut self, offset: usize) -> Token {
        let mut length = self.digits(0);

        if let Some(&(_, '.')) = self.chars.peek(length) {
            length += 1;
            length = self.digits(length);
        }

        if let Some(&(_, 'e' | 'E')) = self.chars.peek(length) {
            length += 1;
            if let Some(&(_, ch @ ('+' | '-' | '\u{2011}'))) = self.chars.peek(length) {
                length += ch.len_utf8();
            }
            length = self.digits(length);
        }

        Token {
            kind: TokenKind::Number,
            length,
            offset,
            id: self.id,
        }
    }

    /// Parse digits 0-9.  Returns the new offset after the parsed digits.
    fn digits(&mut self, offset: usize) -> usize {
        let mut len = offset;
        while let Some(&(_, ch)) = self.chars.peek(len) {
            if !ch.is_ascii_digit() {
                break;
            }

            len += 1;
        }

        len
    }

    /// Parse a character string literal
    fn c_string(&mut self, offset: usize) -> Result<Token> {
        let value = &self.source[offset..];
        let mut length = 1;

        while let Some(&(_, ch)) = self.chars.peek(length) {
            length += ch.len_utf8();

            if ch == '"' {
                if matches!(self.chars.peek(length), Some(&(_, '"'))) {
                    length += 1;
                } else {
                    break;
                }
            }
        }

        let value = &value[..length];
        if !value.ends_with('"') {
            return Err(LexerError::NonTerminatedString {
                offset,
                id: self.id,
            });
        }

        Ok(Token {
            kind: TokenKind::CString,
            length,
            offset,
            id: self.id,
        })
    }

    /// Parse either a b_string or an h_string (binary string or hexadecimal string)
    fn bh_string(&mut self, offset: usize) -> Result<Token> {
        let value = &self.source[offset..];
        let mut length = 1;

        while let Some(&(_, ch)) = self.chars.peek(length) {
            length += ch.len_utf8();

            if ch == '\'' {
                break;
            }
        }

        // 'b' or 'h' suffix
        if let Some(&(_, ch)) = self.chars.peek(length) {
            length += ch.len_utf8();
        }

        // validate the end of the string now, but the content of the string is
        // ignored here, so must be checked later.

        let value = &value[..length];
        if !value.ends_with("'B") && !value.ends_with("'H") {
            return Err(LexerError::NonTerminatedBHString {
                offset,
                id: self.id,
            });
        }

        Ok(Token {
            kind: TokenKind::BHString,
            length,
            offset,
            id: self.id,
        })
    }

    /// Parse an object field reference `&name`
    fn field(&mut self, offset: usize) -> Result<Token> {
        let Some(&(_, ch)) = self.chars.peek(1) else {
            return Err(LexerError::MissingFieldName {
                offset,
                id: self.id,
            });
        };

        if !ch.is_ascii_alphabetic() {
            return Err(LexerError::MissingFieldName {
                offset,
                id: self.id,
            });
        }

        // ident doesn't check the first character, so this will consume the `&`
        let ident = self.identifier(ch, offset);

        match ident.kind {
            TokenKind::ValueRefOrIdent => Ok(Token {
                kind: TokenKind::ValueField,
                ..ident
            }),
            TokenKind::TypeOrModuleRef => Ok(Token {
                kind: TokenKind::TypeField,
                ..ident
            }),
            _ => Err(LexerError::KeywordFieldName {
                offset,
                id: self.id,
            }),
        }
    }

    /// Set the lexer's square bracket mode.
    pub fn set_square_bracket_mode(&mut self, mode: SquareBracketMode) {
        self.square_bracket_mode = mode;
    }

    /// Should keyword parsing be enabled
    pub fn enable_keywords(&mut self, mode: bool) {
        self.enable_keywords = mode;
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

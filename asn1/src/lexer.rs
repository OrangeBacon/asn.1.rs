use std::{
    collections::{HashMap, VecDeque},
    str::CharIndices,
    sync::OnceLock,
};

use crate::{
    compiler::{Features, SourceId},
    diagnostic::{Diagnostic, Label, Result},
    token::{self, Token, TokenKind},
    util::{Peek, Peekable},
    AsnCompiler,
};

/// State for converting a source string into a token stream
#[derive(Debug)]
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

    /// The compiler the lexer was created from.
    features: Features,

    /// All non-fatal diagnostics found during lexing
    diagnostics: Vec<Diagnostic>,
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

impl AsnCompiler {
    /// Create a new Lexer for a given source file.  `file` represents a file
    /// ID that will be returned with each token.
    pub fn lexer(&mut self, id: SourceId) -> Lexer {
        let source = &self.source(id).source;
        Lexer {
            chars: source.char_indices().n_peekable(),
            source,
            id,
            comments: VecDeque::new(),
            square_bracket_mode: Default::default(),
            enable_keywords: true,
            features: self.features,
            diagnostics: vec![],
        }
    }
}

impl<'a> Lexer<'a> {
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

        let loc = self.offset();

        let &(offset, c) = self.chars.peek(0).ok_or_else(|| {
            Diagnostic::error("Asn::Parser::EndOfFile")
                .name("Unexpected end of file")
                .label(Label::new().source(self.id).loc(loc..loc))
        })?;

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

            '"' => self.c_string(offset)?,
            '\'' => self.bh_string(offset)?,

            '0'..='9' => self.number(offset),

            ch if is_ident_start(ch) => self.identifier(c, offset),

            ch => {
                return Err(Diagnostic::error("Asn::Parser::Character")
                    .name("Unexpected character within source file")
                    .label(
                        Label::new()
                            .source(self.id)
                            .loc(offset..offset + ch.len_utf8()),
                    ))
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
        let loc = self.offset();

        let &(offset, ch) = self.chars.peek(0).ok_or_else(|| {
            Diagnostic::error("Asn::Parser::EndOfFile")
                .name("Unexpected end of file")
                .label(Label::new().source(self.id).loc(loc..loc))
                .label("Encountered while parsing an XML value")
        })?;

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
                    length: length.try_into().unwrap(),
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
        self.offset() == self.source.len()
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
                _ if self.is_whitespace(c) => {
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
            length: value.len().try_into().unwrap(),
            offset,
            id: self.id,
        }
    }

    /// Parse a single line comment which is text between pairs of two hyphens.
    /// Non-breaking hyphens are also accepted instead of hyphens.
    fn single_comment(&mut self, first: char, offset: usize) -> bool {
        let Some(&(_, second)) = self.chars.peek(1) else {
            return false;
        };
        if !matches!(second, '-' | '\u{2011}') {
            return false;
        }
        self.chars.next(); // Consume the first hyphen
        self.chars.next(); // Consume the second hyphen

        let mut length = first.len_utf8() + second.len_utf8();

        while let Some(&(_, next)) = self.chars.peek(0) {
            if self.is_newline(next) {
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
            length: length.try_into().unwrap(),
            offset,
            id: self.id,
        });

        true
    }

    /// Parse a multi line comment which is text between `/*` and `*/`.  The comment
    /// ends when a matching `*/` has been found for every `/*` encountered.
    fn multi_comment(&mut self, offset: usize) -> Result<bool> {
        let Some(&(_, c)) = self.chars.peek(1) else {
            return Ok(false);
        };
        if c != '*' {
            // not a start of comment
            return Ok(false);
        }

        self.chars.next();
        self.chars.next();

        let mut length = 2;
        let mut starts = vec![offset];

        while let Some(&(offset, c)) = self.chars.peek(0) {
            if starts.is_empty() {
                break;
            }

            length += c.len_utf8();
            self.chars.next();

            if c == '/' && matches!(self.chars.peek(0), Some((_, '*'))) {
                length += 1;
                starts.push(offset);
                self.chars.next();
            } else if c == '*' && matches!(self.chars.peek(0), Some((_, '/'))) {
                length += 1;
                starts.pop();
                self.chars.next();
            }
        }

        if !starts.is_empty() {
            let mut diag = Diagnostic::error("Asn::Parser::Comment")
                .name("End of file within multi-line comment")
                .label(
                    Label::new()
                        .source(self.id)
                        .loc(self.source.len()..self.source.len())
                        .message(format!(
                            "Add {} closing multi-line comment marker{} `*/`",
                            starts.len(),
                            if starts.len() == 1 { "" } else { "s" }
                        )),
                );

            let mut msg = "Comment started here";
            for offset in starts {
                diag = diag.label(
                    Label::new()
                        .source(self.id)
                        .loc(offset..offset + 2)
                        .message(msg),
                );
                msg = "Nested comment started here"
            }

            return Err(diag);
        }

        self.comments.push_back(Token {
            kind: TokenKind::MultiComment,
            length: length.try_into().unwrap(),
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

        let mut length = first.len_utf8();
        while let Some(&(_, ch)) = self.chars.peek(length) {
            if is_ident_continue(ch) {
                length += ch.len_utf8();
                continue;
            }

            if ch == '-' || ch == '\u{2011}' {
                if let Some(&(_, after)) = self.chars.peek(length + 1) {
                    if is_ident_continue(after) {
                        length += ch.len_utf8();
                        continue;
                    }
                }
            }

            break;
        }

        let ident_kind = if unicode_data::UPPERCASE_LETTER.contains_char(first) {
            TokenKind::TypeOrModuleRef
        } else {
            TokenKind::ValueRefOrIdent
        };

        let value = &value[..length];
        let kind = self.keyword_kind(value, ident_kind);

        if !self.features.unicode_identifiers {
            let mut is_valid = first.is_ascii_alphabetic();
            for ch in value.chars() {
                is_valid |= ch.is_ascii_alphanumeric()
                    || ch == '$'
                    || ch == '_'
                    || ch == '-'
                    || ch == '\u{2011}';
            }

            if !is_valid {
                self.diagnostics
                    .push(Diagnostic::error("Asn1::Parser::UnicodeIdentifier"))
            }
        }

        Token {
            kind,
            length: length.try_into().unwrap(),
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
            length: length.try_into().unwrap(),
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

        let mut double = None;
        while let Some(&(offset, ch)) = self.chars.peek(length) {
            length += ch.len_utf8();

            if ch == '"' {
                if matches!(self.chars.peek(length), Some(&(_, '"'))) {
                    length += 1;

                    if double.is_none() {
                        double = Some(offset);
                    }
                } else {
                    break;
                }
            }
        }

        let value = &value[..length];
        if !value.ends_with('"') {
            let mut diag = Diagnostic::error("Asn::Parser::StringEnd")
                .name("End of file within string literal")
                .label(
                    Label::new()
                        .source(self.id)
                        .loc(self.source.len()..self.source.len())
                        .message("String literal starting here"),
                );
            if let Some(offset) = double {
                diag = diag.label(
                    Label::new()
                        .source(self.id)
                        .loc(offset..offset + 2)
                        .message(
                            "Note that two consecutive double quotes are used \
as an escape sequence for putting a single double quote in the string literal, \
this does not represent two adjacent strings.",
                        ),
                );
            }
            return Err(diag);
        }

        Ok(Token {
            kind: TokenKind::CString,
            length: length.try_into().unwrap(),
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
            if value.ends_with('\'') {
                return Err(Diagnostic::error("Asn::Parser::DataString")
                    .name("Missing radix after data string")
                    .label(
                        Label::new()
                            .source(self.id)
                            .loc(offset + length - 1..offset + length),
                    ));
            } else {
                return Err(Diagnostic::error("Asn::Parser::DataString")
                    .name("Missing ending of data string")
                    .label(
                        Label::new()
                            .source(self.id)
                            .loc(offset + length..offset + length),
                    ));
            }
        }

        Ok(Token {
            kind: TokenKind::BHString,
            length: length.try_into().unwrap(),
            offset,
            id: self.id,
        })
    }

    /// Parse an object field reference `&name`
    fn field(&mut self, offset: usize) -> Result<Token> {
        let Some(&(_, ch)) = self.chars.peek(1) else {
            let loc = self.offset();
            return Err(Diagnostic::error("Asn::Parser::Field")
                .name("Unexpected end of file after `&`")
                .label(Label::new().source(self.id).loc(loc..loc)));
        };

        if !is_ident_start(ch) {
            return Err(Diagnostic::error("Asn::Parser::Field")
                .name("Expected identifier after `&`")
                .label(
                    Label::new()
                        .source(self.id)
                        .loc(offset + 1..offset + ch.len_utf8())
                        .message("This character cannot start an identifier"),
                ));
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
            _ => Err(Diagnostic::error("Asn::Parser::Field")
                .name("Unexpected keyword after `&`")
                .label(
                    Label::new()
                        .source(self.id)
                        .loc(ident.offset..ident.offset + (ident.length as usize))
                        .message("Expected a non-keyword identifier here"),
                )),
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

    /// Is the character any valid whitespace
    fn is_whitespace(&self, c: char) -> bool {
        // A0 = Non breaking space
        let unicode =
            self.features.unicode_whitespace && unicode_data::WHITE_SPACE.contains_char(c);
        "\t \u{A0}".contains(c) || self.is_newline(c) || unicode
    }

    /// Is the character a valid newline character.  Do not use for calculating line
    /// number as this does not count CRLF as one line end.
    fn is_newline(&self, c: char) -> bool {
        // 0B = Vertical Tab
        // 0C = Form Feed
        let unicode = self.features.unicode_whitespace
            && "\u{A}\u{B}\u{C}\u{D}\u{85}\u{2028}\u{2029}".contains(c);
        "\n\x0B\x0C\r".contains(c) || unicode
    }

    /// Match an identifier to a keyword if possible, otherwise return the provided
    /// default value.
    fn keyword_kind(&self, value: &str, ident_kind: TokenKind) -> TokenKind {
        if self.enable_keywords {
            if let Some(kw) = keywords().get(value).copied() {
                return kw;
            } else if let Some(kw) = lower_keywords().get(value).copied() {
                if self.features.lowercase_keywords {
                    return kw;
                }
            }
        }

        ident_kind
    }
}

/// Get a mapping from keyword strings to their token kind
fn keywords() -> &'static HashMap<&'static str, TokenKind> {
    static KEYWORDS: OnceLock<HashMap<&'static str, TokenKind>> = OnceLock::new();
    KEYWORDS.get_or_init(|| HashMap::from(token::KEYWORD_DATA.map(|(a, b, _)| (a, b))))
}

/// Get a mapping from keyword strings to their token kind
fn lower_keywords() -> &'static HashMap<&'static str, TokenKind> {
    static KEYWORDS: OnceLock<HashMap<&'static str, TokenKind>> = OnceLock::new();
    KEYWORDS.get_or_init(|| HashMap::from(token::KEYWORD_DATA.map(|(_, b, a)| (a, b))))
}

fn is_ident_start(ch: char) -> bool {
    ch == '$' || ch == '_' || ch.is_ascii_alphabetic() || unicode_data::XID_START.contains_char(ch)
}

fn is_ident_continue(ch: char) -> bool {
    ch == '$'
        || ch == '_'
        || ch == '-'
        || ch == '\u{2011}'
        || ch.is_ascii_alphanumeric()
        || unicode_data::XID_START.contains_char(ch)
}

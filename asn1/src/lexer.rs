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
        offset: Option<usize>,
        file: usize,
    },

    NonTerminatedComment {
        offset: usize,
        file: usize,
    },
}

pub type Result<T, E = LexerError> = std::result::Result<T, E>;

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

    /// Return a token of one of the provided token kinds.  If none could be matched,
    /// returns an error saying what was expected and what was found.  If there
    /// are no more characters in the source text, returns an error.  Skips all
    /// whitespace and comments before the start of a token.
    pub fn peek(&mut self, kind: &'static [TokenKind]) -> Result<Token<'a>> {
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

        let &(offset, c) = self.chars.peek(0).ok_or(LexerError::Expected {
            kind,
            offset: None,
            file: self.file,
        })?;

        for &kind in kind {
            let ret = match kind {
                TokenKind::LeftCurly if c == '{' => self.simple_token(kind, offset),
                TokenKind::RightCurly if c == '}' => self.simple_token(kind, offset),
                TokenKind::Less if c == '<' => self.simple_token(kind, offset),
                TokenKind::Greater if c == '>' => self.simple_token(kind, offset),
                TokenKind::Comma if c == ',' => self.simple_token(kind, offset),
                TokenKind::Dot if c == '.' => self.simple_token(kind, offset),
                TokenKind::ForwardSlash if c == '/' => self.simple_token(kind, offset),
                TokenKind::LeftParen if c == '(' => self.simple_token(kind, offset),
                TokenKind::RightParen if c == ')' => self.simple_token(kind, offset),
                TokenKind::LeftSquare if c == '[' => self.simple_token(kind, offset),
                TokenKind::RightSquare if c == ']' => self.simple_token(kind, offset),
                TokenKind::Hyphen if c == '-' || c == '\u{2011}' => self.simple_token(kind, offset),
                TokenKind::Colon if c == ':' => self.simple_token(kind, offset),
                TokenKind::Equals if c == '=' => self.simple_token(kind, offset),
                TokenKind::DoubleQuote if c == '"' => self.simple_token(kind, offset),
                TokenKind::SemiColon if c == ';' => self.simple_token(kind, offset),
                TokenKind::At if c == '@' => self.simple_token(kind, offset),
                TokenKind::Pipe if c == '|' => self.simple_token(kind, offset),
                TokenKind::Exclamation if c == '!' => self.simple_token(kind, offset),
                TokenKind::Caret if c == '^' => self.simple_token(kind, offset),
                TokenKind::Underscore if c == '_' => self.simple_token(kind, offset),

                TokenKind::Assignment if c == ':' => {
                    self.multi_token(TokenKind::Assignment, offset, "::=")
                }
                TokenKind::XMLEndTag if c == '<' => {
                    self.multi_token(TokenKind::XMLEndTag, offset, "</")
                }
                TokenKind::XMLSingleTagEnd if c == '/' => {
                    self.multi_token(TokenKind::XMLSingleTagEnd, offset, "/>")
                }

                TokenKind::Identifier | TokenKind::ValueReference if c.is_ascii_alphabetic() => {
                    let ident = self.identifier(c, offset);

                    if ident.kind == TokenKind::Identifier {
                        Some(Token { kind, ..ident })
                    } else {
                        None
                    }
                }
                TokenKind::TypeReference | TokenKind::ModuleReference if c.is_ascii_alphabetic() => {
                    let ident = self.identifier(c, offset);

                    if ident.kind == TokenKind::TypeReference || ident.kind == TokenKind::EncodingReference {
                        Some(Token { kind, ..ident })
                    } else {
                        None
                    }
                }
                TokenKind::EncodingReference if c.is_ascii_alphabetic() => {
                    let ident = self.identifier(c, offset);

                    if ident.kind == TokenKind::EncodingReference {
                        Some(ident)
                    } else {
                        None
                    }
                }
                TokenKind::XMLAsn1TypeName if c.is_ascii_alphabetic() => {
                    let ident = self.identifier(c, offset);

                    Some(Token { kind, ..ident })
                }

                TokenKind::IntegerUnicodeLabel => self.integer_unicode_label(c, offset),
                TokenKind::NonIntegerUnicodeLabel => self.non_integer_unicode_label(c, offset),
                TokenKind::Number => self.number(c, offset),

                TokenKind::IdentTrue if c.is_ascii_alphabetic() => {
                    let ident = self.identifier(c, offset);

                    if ident.value == "true" {
                        Some(Token { kind:TokenKind::IdentTrue,.. ident})
                    } else {
                        None
                    }
                }
                TokenKind::IdentFalse if c.is_ascii_alphabetic() => {
                    let ident = self.identifier(c, offset);

                    if ident.value == "false" {
                        Some(Token { kind:TokenKind::IdentTrue,.. ident})
                    } else {
                        None
                    }
                }
                TokenKind::XMLBoolNumber if c == '0' || c == '1' => {
                    self.number(c, offset)
                        .filter(|&tok| tok.value == "0" || tok.value == "1")
                        .map(|t| Token { kind:TokenKind::XMLBoolNumber, ..t })
                }

                // "ABSENT",
                // "ABSTRACT-SYNTAX",
                // "ALL",
                // "APPLICATION",
                TokenKind::KwAutomatic
                | TokenKind::KwBegin
                // "BIT",
                // "BMPString",
                | TokenKind::KwBoolean
                // "BY",
                // "CHARACTER",
                // "CHOICE",
                // "CLASS",
                // "COMPONENT",
                // "COMPONENTS",
                // "CONSTRAINED",
                // "CONTAINING",
                // "DATE",
                // "DATE-TIME",
                // "DEFAULT",
                | TokenKind::KwDefinitions
                // "DURATION",
                // "EMBEDDED",
                // "ENCODED",
                // "ENCODING-CONTROL",
                | TokenKind::KwEnd
                // "ENUMERATED",
                // "EXCEPT",
                | TokenKind::KwExplicit
                // "EXPORTS",
                | TokenKind::KwExtensibility
                // "EXTERNAL",
                | TokenKind::KwFalse
                // "FROM",
                // "GeneralizedTime",
                // "GeneralString",
                // "IA5String",
                // "IDENTIFIER",
                | TokenKind::KwImplicit
                | TokenKind::KwImplied
                // "IMPORTS",
                // "INCLUDES",
                // "INSTANCE",
                | TokenKind::KwInstructions
                | TokenKind::KwInteger
                // "INTERSECTION",
                // "ISO646String",
                // "MAX",
                // "MIN",
                // "MINUS-INFINITY",
                // "NOT-A-NUMBER",
                | TokenKind::KwNull
                // "NumericString",
                // "OBJECT",
                // "ObjectDescriptor",
                // "OCTET",
                // "OF",
                | TokenKind::KwOidIri
                // "OPTIONAL",
                // "PATTERN",
                // "PDV",
                // "PLUS-INFINITY",
                // "PRESENT",
                // "PrintableString",
                // "PRIVATE",
                // "REAL",
                // "RELATIVE-OID",
                // "RELATIVE-OID-IRI",
                // "SEQUENCE",
                // "SET",
                // "SETTINGS",
                // "SIZE",
                // "STRING",
                // "SYNTAX",
                // "T61String",
                | TokenKind::KwTags
                // "TeletexString",
                // "TIME",
                // "TIME-OF-DAY",
                | TokenKind::KwTrue
                // "TYPE-IDENTIFIER",
                // "UNION",
                // "UNIQUE",
                // "UNIVERSAL",
                // "UniversalString",
                // "UTCTime",
                // "UTF8String",
                // "VideotexString",
                // "VisibleString",
                // "WITH",
                if c.is_ascii_alphabetic() => {
                    let ident = self.identifier(c, offset);

                    if ident.kind == kind {
                        Some(ident)
                    } else {
                        None
                    }
                },

                _ => None,
            };

            if let Some(ret) = ret {
                return Ok(ret);
            }
        }

        Err(LexerError::Expected {
            kind,
            offset: Some(offset),
            file: self.file,
        })
    }

    /// Peeks a token of the given kind (see peek()), then advances the source
    /// text past the token.  Might also return a comment token instead.
    pub fn next(&mut self, kind: &'static [TokenKind]) -> Result<Token<'a>> {
        let peek = self.peek(kind);

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
        match self.peek(&[]) {
            Ok(_) => panic!("Managed to successfully lex nothing?"),
            Err(LexerError::Expected { offset: None, .. }) => true,
            _ => false,
        }
    }

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

        Some(Token {
            kind,
            value: &tok_value[..value.len()],
            offset,
            file: self.file,
        })
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
        let mut contains_lower = first.is_ascii_lowercase();
        while let Some(&(_, c)) = self.chars.peek(len) {
            if c.is_ascii_alphanumeric() || "$_".contains(c) {
                contains_lower |= c.is_ascii_lowercase();
                len += 1;
                continue;
            }

            if c == '-' || c == '\u{2011}' {
                if let Some(&(_, c)) = self.chars.peek(len + 1) {
                    if c.is_ascii_alphanumeric() {
                        // does not check the hyphen as it does not count as
                        // lower or upper case
                        contains_lower |= c.is_ascii_lowercase();
                        len += 2;
                        continue;
                    }
                }
            }

            break;
        }

        let ident_kind = if !contains_lower {
            TokenKind::EncodingReference
        } else if first.is_ascii_lowercase() {
            TokenKind::Identifier
        } else {
            TokenKind::TypeReference
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

    /// Parse an integer valued unicode label as per X.680 (02/2021) 12.26
    fn integer_unicode_label(&mut self, first: char, offset: usize) -> Option<Token<'a>> {
        if !first.is_ascii_digit() {
            return None;
        }

        let value = &self.source[offset..];
        let mut len = 1;
        let mut is_number = true;
        while let Some(&(_, ch)) = self.chars.peek(len) {
            is_number &= ch.is_ascii_digit();

            if !is_unicode_label_char(ch) {
                break;
            }

            len += ch.len_utf8();
        }

        let value = &value[..len];
        if value == "0" || !is_number {
            return None;
        }

        Some(Token {
            kind: TokenKind::IntegerUnicodeLabel,
            value,
            offset,
            file: self.file,
        })
    }

    /// Parse an non-integer unicode label as per X.680 (02/2021) 12.27
    fn non_integer_unicode_label(&mut self, first: char, offset: usize) -> Option<Token<'a>> {
        if !is_unicode_label_char(first) || first == '-' || first == '\u{2011}' {
            return None;
        }

        let value = &self.source[offset..];
        let mut len = 1;
        let mut is_number = true;
        while let Some(&(_, ch)) = self.chars.peek(len) {
            is_number &= ch.is_ascii_digit();

            if !is_unicode_label_char(ch) {
                break;
            }

            len += ch.len_utf8();
        }

        let value = &value[..len];

        let mut chars = value.chars();
        let third = chars.nth(2);
        let fourth = chars.next();
        if is_number
            || (matches!(third, Some('-' | '\u{2011}')) && matches!(fourth, Some('-' | '\u{2011}')))
        {
            return None;
        }

        Some(Token {
            kind: TokenKind::NonIntegerUnicodeLabel,
            value,
            offset,
            file: self.file,
        })
    }

    /// Parse a number ([1-9][0-9]*)|0
    fn number(&mut self, first: char, offset: usize) -> Option<Token<'a>> {
        if !first.is_ascii_digit() {
            return None;
        }

        let value = &self.source[offset..];
        let mut len = 1;
        while let Some(&(_, ch)) = self.chars.peek(len) {
            if !ch.is_ascii_digit() {
                break;
            }

            len += 1;
        }

        let value = &value[..len];
        if value.starts_with('0') && len > 1 {
            return None;
        }

        Some(Token {
            kind: TokenKind::Number,
            value,
            offset,
            file: self.file,
        })
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

/// Is the character allowed to be in a non-integer unicode label as per
/// X.660 (07/2011) 7.5.2, however that includes some of the utf-16 surrogates
/// (but not all? idk?) so the definition below that has been taken from
/// rfc 3987 (January 2005) section 2.2, rule "iunreserved"
fn is_unicode_label_char(c: char) -> bool {
    "-._~".contains(c)
        || c.is_ascii_alphanumeric()
        // note: using U+D7FF
        || ('\u{000A0}'..='\u{0D7FF}').contains(&c)
        || ('\u{0F900}'..'\u{0FDCF}').contains(&c)
        || ('\u{0FDF0}'..'\u{0FFEF}').contains(&c)
        || ('\u{0FDF0}'..'\u{0FFEF}').contains(&c)
        || ('\u{10000}'..'\u{1FFFD}').contains(&c)
        || ('\u{20000}'..'\u{2FFFD}').contains(&c)
        || ('\u{30000}'..'\u{3FFFD}').contains(&c)
        || ('\u{40000}'..'\u{4FFFD}').contains(&c)
        || ('\u{50000}'..'\u{5FFFD}').contains(&c)
        || ('\u{60000}'..'\u{6FFFD}').contains(&c)
        || ('\u{70000}'..'\u{7FFFD}').contains(&c)
        || ('\u{80000}'..'\u{8FFFD}').contains(&c)
        || ('\u{90000}'..'\u{9FFFD}').contains(&c)
        || ('\u{A0000}'..'\u{AFFFD}').contains(&c)
        || ('\u{B0000}'..'\u{BFFFD}').contains(&c)
        || ('\u{C0000}'..'\u{CFFFD}').contains(&c)
        || ('\u{D0000}'..'\u{DFFFD}').contains(&c)
        || ('\u{E1000}'..'\u{EFFFD}').contains(&c)
}

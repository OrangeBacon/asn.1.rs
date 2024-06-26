use std::{collections::HashMap, str::CharIndices, sync::OnceLock};

use itertools::{Itertools, PeekNth};
use unicode_normalization::{is_nfkc_quick, IsNormalized};

use crate::token::{self, Token, TokenKind, TokenKindFlags};

pub struct Lexer<'a> {
    /// Iterator over all chars in the file
    chars: PeekNth<CharIndices<'a>>,

    /// The original source text
    source: &'a str,
}

/// Is the provided character supposed to be part of a comment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum IsComment {
    Yes,
    No,
}

impl<'a> Lexer<'a> {
    /// Get all tokens for a source file.  Note that pragmas are described in the
    /// lexical syntax of the ada reference manual, however this lexer treats them
    /// as something to be handled by the parser.  All possible errors are returned
    /// inline with the token stream.
    pub fn run(source: &'a str) -> Vec<Token> {
        // reject non-nfc source code
        if !unicode_normalization::is_nfc(source) {
            return vec![Token {
                kind: TokenKind::NfcError,
                start: 0,
                end: source.len(),
            }];
        }

        // skip byte order mark
        let source = source.strip_prefix('\u{FEFF}').unwrap_or(source);

        let mut lexer = Lexer {
            chars: itertools::peek_nth(source.char_indices()),
            source,
        };

        let mut tokens = vec![];
        loop {
            let (Ok(tok) | Err(tok)) = lexer.next();
            if tok.kind == TokenKind::Eof {
                break;
            }
            if let Some(last) = tokens.last_mut() {
                if !join_tokens(last, tok) {
                    tokens.push(tok);
                }
            } else {
                tokens.push(tok);
            }
        }

        identifier_separator(source, tokens)
    }

    /// Get the next available token.  Note that the return type is a result, however
    /// a token might be returned as either the Ok or Err variant, which should not
    /// make a difference for the lexer, it is just to allow the `?` operator to be
    /// used.
    fn next(&mut self) -> Result<Token, Token> {
        let (loc, ch) = self.ch()?;
        self.is_valid(ch, loc, IsComment::No)?;

        // helper for simple tokens
        let t = |k: TokenKind| Token {
            kind: k,
            start: loc,
            end: loc + ch.len_utf8(),
        };

        Ok(match ch {
            '&' => t(TokenKind::Ampersand),
            '\'' => t(TokenKind::Apostrophe),
            '(' => t(TokenKind::LParen),
            ')' => t(TokenKind::RParen),
            '+' => t(TokenKind::Plus),
            ',' => t(TokenKind::Comma),
            ';' => t(TokenKind::SemiColon),
            '@' => t(TokenKind::At),
            '[' => t(TokenKind::LSquare),
            ']' => t(TokenKind::RSquare),
            '|' => t(TokenKind::VerticalBar),

            '=' => self.multi_token(t(TokenKind::Equals), &[('>', TokenKind::Arrow)])?,
            '.' => self.multi_token(t(TokenKind::Dot), &[('.', TokenKind::DoubleDot)])?,
            '*' => self.multi_token(t(TokenKind::Star), &[('*', TokenKind::DoubleStar)])?,
            ':' => self.multi_token(t(TokenKind::Colon), &[('=', TokenKind::ColonEquals)])?,
            '/' => self.multi_token(t(TokenKind::Slash), &[('=', TokenKind::SlashEquals)])?,
            '>' => self.multi_token(
                t(TokenKind::GreaterThan),
                &[
                    ('=', TokenKind::GreaterEquals),
                    ('>', TokenKind::GreaterGreater),
                ],
            )?,
            '<' => self.multi_token(
                t(TokenKind::LessThan),
                &[
                    ('=', TokenKind::LessEquals),
                    ('<', TokenKind::LessLess),
                    ('>', TokenKind::Box),
                ],
            )?,

            '0'..='9' => self.number(loc, ch),
            '"' => self.string(loc, ch),
            '-' => self.comment(loc, ch),
            _ if is_ident_start(ch) => self.ident(loc, ch),
            _ if is_whitespace(ch) => t(TokenKind::Whitespace),
            _ => t(TokenKind::Error),
        })
    }

    /// Check the next character after a token to see if a multi-character token
    /// should be emitted instead
    fn multi_token(&mut self, t: Token, value: &[(char, TokenKind)]) -> Result<Token, Token> {
        let Some(&(_, ch)) = self.chars.peek() else {
            return Ok(t);
        };

        for &val in value {
            if val.0 == ch {
                // we know the char is valid as we specified the char in the input array
                self.chars.next();

                return Ok(Token {
                    kind: val.1,
                    start: t.start,
                    end: t.end + val.0.len_utf8(),
                });
            }
        }

        Ok(t)
    }

    /// Parse an identifier token
    fn ident(&mut self, start: usize, ch: char) -> Token {
        let mut end = start + ch.len_utf8();
        let mut is_err = false;
        let mut is_last_punc = false;

        while let Some(&(loc, ch)) = self.chars.peek() {
            if !is_ident(ch) {
                break;
            }

            self.chars.next();
            end += ch.len_utf8();

            // All characters must be valid for being in NFKC and valid for being
            // in a source file
            if is_nfkc_quick(std::iter::once(ch)) == IsNormalized::No
                || self.is_valid(ch, loc, IsComment::No).is_err()
            {
                is_err = true;
            }

            // No 2 adjacent punctuation characters allowed
            if unicode_data::CONNECTOR_PUNCTUATION.contains_char(ch) {
                if is_last_punc {
                    is_err = true
                }
                is_last_punc = true;
            } else {
                is_last_punc = false;
            }
        }

        if is_err || is_last_punc {
            return Token {
                kind: TokenKind::IdentifierError,
                start,
                end,
            };
        }

        let text = case_fold(&self.source[start..end]);
        let kind = keyword_table()
            .get(&text)
            .copied()
            .unwrap_or(TokenKind::Identifier);

        Token { kind, start, end }
    }

    /// Parse either a comment or a '-' symbol
    fn comment(&mut self, start: usize, ch: char) -> Token {
        let mut end = start + ch.len_utf8();

        if !matches!(self.chars.peek(), Some((_, '-'))) {
            return Token {
                kind: TokenKind::Hyphen,
                start,
                end: start + ch.len_utf8(),
            };
        }

        let mut is_err = false;
        while let Some(&(loc, ch)) = self.chars.peek() {
            if is_end_of_line(ch) {
                break;
            }

            self.chars.next();
            end += ch.len_utf8();

            if self.is_valid(ch, loc, IsComment::Yes).is_err() {
                is_err = true;
            }
        }

        let kind = if is_err {
            TokenKind::CommentError
        } else {
            TokenKind::Comment
        };

        Token { kind, start, end }
    }

    /// Parse a string literal
    fn string(&mut self, start: usize, ch: char) -> Token {
        let mut end = start + ch.len_utf8();

        let mut is_err = false;
        while let Some((loc, ch)) = self.chars.next() {
            end += ch.len_utf8();

            if ch == '"' {
                if self.chars.next_if(|&(_, b)| b == '"').is_some() {
                    end += '"'.len_utf8();
                } else {
                    break;
                }
            }
            if !is_graphic_character(ch) {
                is_err = true;
            }
            if self.is_valid(ch, loc, IsComment::No).is_err() {
                is_err = true;
            }
        }

        let kind = if is_err || !self.source[start..end].ends_with('"') {
            TokenKind::StringError
        } else {
            TokenKind::String
        };

        Token { kind, start, end }
    }

    /// Parse any kind of numeric literal
    fn number(&mut self, start: usize, ch: char) -> Token {
        // ignores validity checking as anything invalid would always end the
        // number literal and leave the validity for the next token.
        let mut is_ok = true;

        let mut end = start + ch.len_utf8();
        self.numeral(&mut end, false);

        // either decimal part or # numeral . numeral #
        let next = self.chars.peek();
        if let Some(&(_, ch @ '.')) = next {
            if matches!(self.chars.peek_nth(1), Some(&(_, '0'..='9'))) {
                self.chars.next();
                end += ch.len_utf8();
                self.numeral(&mut end, false);
            }
        } else if let Some(&(_, ch @ '#')) = next {
            // assumes that `[0-9]#` will always start a based number, rather than
            // being a number, then an un related `#`, allowing for better error
            // messages about missing the second `#` in a number.
            self.chars.next();
            end += ch.len_utf8();
            self.numeral(&mut end, true);

            if let Some(&(_, ch @ '.')) = self.chars.peek() {
                self.chars.next();
                end += ch.len_utf8();

                if let Some(&(_, ch)) = self.chars.peek() {
                    is_ok &= ch.is_ascii_hexdigit();
                    self.numeral(&mut end, true);
                }
            }

            if let Some(&(_, ch @ '#')) = self.chars.peek() {
                self.chars.next();
                end += ch.len_utf8();
            } else {
                is_ok = false;
            }
        }

        // exponent or nothing
        if let Some(&(_, ee @ ('e' | 'E'))) = self.chars.peek() {
            let next = self.chars.peek_nth(1).map(|(_, b)| *b);
            if let Some(pm @ ('+' | '-')) = next {
                if matches!(self.chars.peek_nth(2), Some(&(_, '0'..='9'))) {
                    self.chars.next(); // skip the `[eE]`
                    self.chars.next(); // skip the `[+-]`
                    end += ee.len_utf8() + pm.len_utf8();
                    self.numeral(&mut end, false);
                }
            } else if matches!(next, Some('0'..='9')) {
                self.chars.next(); // skip the `[eE]`
                end += ee.len_utf8();
                self.numeral(&mut end, false);
            }
        }

        let kind = if is_ok {
            TokenKind::Number
        } else {
            TokenKind::NumberError
        };

        Token { kind, start, end }
    }

    /// Parse a numeral portion of a number literal, regex = `[0-9](_?[0-9])*`.
    /// Optionally, if `is_hex` is true, allows hexadecimal digits `[a-fA-F]` in
    /// addition to `[0-9]`.
    fn numeral(&mut self, end: &mut usize, is_hex: bool) {
        let check = if is_hex {
            char::is_ascii_hexdigit
        } else {
            char::is_ascii_digit
        };

        while let Some(&(_, ch)) = self.chars.peek() {
            if check(&ch) {
                self.chars.next();
                *end += ch.len_utf8();
            } else if ch == '_' {
                if let Some(&(_, next)) = self.chars.peek_nth(1) {
                    if check(&next) {
                        self.chars.next();
                        self.chars.next();
                        *end += ch.len_utf8() + next.len_utf8();
                    }
                }
            } else {
                break;
            }
        }
    }

    /// Check if the provided character is valid to be in a source file, otherwise
    /// return an error token representing the character.  `loc` is the index of the
    /// character within the source file to use for error reporting.  If the character
    /// is a member of a comment, then the restrictions are partially relaxed.
    fn is_valid(&mut self, ch: char, loc: usize, is_comment: IsComment) -> Result<char, Token> {
        // characters that are at position 0xFFFE or 0xFFFF within their plane
        // are invalid, so should be rejected.  Surrogate code points are already
        // rejected by rust not allowing them.
        let plane = ch as u32 % 2u32.pow(16);
        if plane == 0xFFFE || plane == 0xFFFF {
            return Err(Token {
                kind: TokenKind::UnicodeError,
                start: loc,
                end: loc + ch.len_utf8(),
            });
        }

        if is_comment == IsComment::Yes
            || unicode_data::FORMAT.contains_char(ch)
            || is_format_effector(ch)
            || is_graphic_character(ch)
        {
            Ok(ch)
        } else {
            Err(Token {
                kind: TokenKind::UnicodeNotCommentError,
                start: loc,
                end: loc + ch.len_utf8(),
            })
        }
    }

    /// Consume and return the next character from the source text
    fn ch(&mut self) -> Result<(usize, char), Token> {
        let Some((start, ch)) = self.chars.next() else {
            return Err(Token {
                kind: TokenKind::Eof,
                start: self.source.len(),
                end: self.source.len(),
            });
        };

        Ok((start, ch))
    }
}

/// Attempt to join two adjacent tokens into one token.  If successful, modifies
/// `last` to be the extended token and returns true, otherwise does not change
/// `last` and returns false.
fn join_tokens(last: &mut Token, new: Token) -> bool {
    assert_eq!(last.end, new.start);

    if last.kind != new.kind {
        return false;
    }

    use TokenKind::*;
    let to_join = NfcError | UnicodeError | UnicodeNotCommentError | Error | Eof | Whitespace;

    if to_join.contains(last.kind) {
        last.end = new.end
    }

    true
}

/// Ensure that separators are present between any two reserved words, identifiers
/// or number literals by modifying a token stream.
fn identifier_separator(source: &str, mut tokens: Vec<Token>) -> Vec<Token> {
    let identifier_kinds = identifier_kinds();

    // check whitespace contains a valid separator
    let mut edits = vec![];
    for ((_, left), (idx, mid), (_, right)) in tokens.iter().copied().enumerate().tuple_windows() {
        if !(identifier_kinds.contains(left.kind)
            && mid.kind == TokenKind::Whitespace
            && identifier_kinds.contains(right.kind))
        {
            continue;
        }

        let contains_sep = source[mid.start..mid.end].chars().any(is_separator);
        if !contains_sep {
            edits.push(idx);
        }
    }
    // set error tokens if required
    for edit in edits {
        tokens[edit].kind = TokenKind::SeparatorError;
    }

    // check two identifiers are not next to each other without a separator
    let mut edits = vec![];
    for ((_, left), (idx, right)) in tokens.iter().copied().enumerate().tuple_windows() {
        if identifier_kinds.contains(left.kind) && identifier_kinds.contains(right.kind) {
            edits.push((idx, left.end));
        }
    }
    tokens.reserve(edits.len());
    for (idx, loc) in edits.into_iter().rev() {
        tokens.insert(
            idx,
            Token {
                kind: TokenKind::SeparatorError,
                start: loc,
                end: loc,
            },
        )
    }

    tokens
}

/// Is a character classified as a format effector
fn is_format_effector(ch: char) -> bool {
    "\t\n\u{0C}\r\u{85}".contains(ch)
        || unicode_data::LINE_SEPARATOR.contains_char(ch)
        || unicode_data::PARAGRAPH_SEPARATOR.contains_char(ch)
}

/// Does a character have the right general category to be a graphic character
fn is_graphic_character(ch: char) -> bool {
    !(unicode_data::CONTROL.contains_char(ch)
        || unicode_data::PRIVATE_USE.contains_char(ch)
        || unicode_data::SURROGATE.contains_char(ch)
        || is_format_effector(ch))
}

/// Does the character count as a separator between adjacent identifiers
fn is_separator(ch: char) -> bool {
    unicode_data::SPACE_SEPARATOR.contains_char(ch) || is_format_effector(ch)
}

/// Is the character whitespace, that is allowed anywhere
fn is_whitespace(ch: char) -> bool {
    is_separator(ch) || unicode_data::FORMAT.contains_char(ch)
}

/// Is the character allowed at the start of an identifier
fn is_ident_start(ch: char) -> bool {
    unicode_data::UPPERCASE_LETTER.contains_char(ch)
        || unicode_data::LOWERCASE_LETTER.contains_char(ch)
        || unicode_data::TITLECASE_LETTER.contains_char(ch)
        || unicode_data::MODIFIER_LETTER.contains_char(ch)
        || unicode_data::OTHER_LETTER.contains_char(ch)
        || unicode_data::LETTER_NUMBER.contains_char(ch)
}

/// Is the character allowed anywhere within an identifier
fn is_ident(ch: char) -> bool {
    is_ident_start(ch)
        || unicode_data::NONSPACING_MARK.contains_char(ch)
        || unicode_data::SPACING_MARK.contains_char(ch)
        || unicode_data::DECIMAL_NUMBER.contains_char(ch)
        || unicode_data::CONNECTOR_PUNCTUATION.contains_char(ch)
}

/// Is the character part of an end of line sequence
fn is_end_of_line(ch: char) -> bool {
    (ch != '\t' && is_format_effector(ch)) || ch == '\r' || ch == '\n'
}

/// Get the keyword lookup table
fn keyword_table() -> &'static HashMap<String, TokenKind> {
    static TABLE: OnceLock<HashMap<String, TokenKind>> = OnceLock::new();

    TABLE.get_or_init(|| {
        token::KEYWORDS
            .iter()
            .map(|&(s, kind)| (case_fold(s), kind))
            .collect()
    })
}

/// Get the token kinds that are valid identifiers, reserved words or numbers.
fn identifier_kinds() -> TokenKindFlags {
    static FLAGS: OnceLock<TokenKindFlags> = OnceLock::new();

    *FLAGS.get_or_init(|| {
        let kind = TokenKind::Number | TokenKind::Identifier;
        token::KEYWORDS
            .iter()
            .map(|&(_, x)| x)
            .fold(kind, |a, b| a | b)
    })
}

/// Case fold a string, for case-insensitive matching
fn case_fold(s: &str) -> String {
    let table = case_fold_table();

    s.chars()
        .map(|c| table.get(&c).copied().unwrap_or(c))
        .collect()
}

/// Get the case folding mapping table
fn case_fold_table() -> &'static HashMap<char, char> {
    static TABLE: OnceLock<HashMap<char, char>> = OnceLock::new();
    TABLE.get_or_init(|| {
        unicode_data::CASE_FOLDING_SIMPLE
            .iter()
            .map(|&(a, b)| (char::from_u32(a).unwrap(), char::from_u32(b).unwrap()))
            .collect()
    })
}

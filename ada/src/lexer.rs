use std::{collections::HashMap, iter::Peekable, str::CharIndices, sync::OnceLock};

use unicode_normalization::{is_nfkc_quick, IsNormalized};

use crate::token::{self, Token, TokenKind};

pub struct Lexer<'a> {
    /// Iterator over all chars in the file
    chars: Peekable<CharIndices<'a>>,

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
    /// Get all tokens for a source file
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
            chars: source.char_indices().peekable(),
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

        tokens
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

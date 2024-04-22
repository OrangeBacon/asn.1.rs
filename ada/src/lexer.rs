use std::{iter::Peekable, str::CharIndices};

use crate::token::{Token, TokenKind};

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

        Ok(match ch {
            _ if is_whitespace(ch) => Token {
                kind: TokenKind::Whitespace,
                start: loc,
                end: loc + ch.len_utf8(),
            },
            ch => Token {
                kind: TokenKind::Error,
                start: loc,
                end: loc + ch.len_utf8(),
            },
        })
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

    false
}

/// Is a character classified as a format effector
fn is_format_effector(ch: char) -> bool {
    "\u{09}\u{0A}\u{0C}\u{0D}\u{85}".contains(ch)
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

fn is_separator(ch: char) -> bool {
    unicode_data::SPACE_SEPARATOR.contains_char(ch) || is_format_effector(ch)
}

fn is_whitespace(ch: char) -> bool {
    is_separator(ch) || unicode_data::FORMAT.contains_char(ch)
}

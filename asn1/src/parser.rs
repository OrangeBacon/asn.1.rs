use std::{
    collections::{HashMap, VecDeque},
    sync::OnceLock,
};

use crate::{
    ast::{Asn1, Asn1Tag, TreeContent},
    lexer::Lexer,
    token::{self, Token, TokenBuffer, TokenKind},
};

/// Parser for ASN.1 definition files
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Parser<'a> {
    /// All tokens from the lexer
    tokens: VecDeque<Token<'a>>,

    /// The partial tree constructed by the parser
    result: Vec<TreeContent<'a>>,

    /// Temporary storage used when making the tree
    temp_result: Vec<TreeContent<'a>>,
}

/// All errors that can be reported by the parser
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ParseError {
    /// Lexer error tokens
    Token(TokenBuffer),

    /// expected token of kind `kind`, got token ... or EOF if None
    Expected(&'static [TokenKind], Option<TokenBuffer>),

    /// A list of errors encountered while parsing
    Multiple(Vec<ParseError>),
}

type Result<T, E = ParseError> = std::result::Result<T, E>;

impl<'a> Parser<'a> {
    /// Create a new parser from a lexer
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self {
            tokens: lexer.collect(),
            result: vec![],
            temp_result: vec![],
        }
    }

    /// Run the parser to produce a set of ASN.1 definitions
    pub fn run(mut self) -> Result<Asn1<'a>> {
        while self.peek(0).is_some() {
            self.module_definition()?;
        }

        // handle comments at the end of the file after all meaningful tokens
        self.next();

        self.end_temp_vec(0, Asn1Tag::Root);
        let root = self.result.len();
        self.result.push(self.temp_result[0]);

        Ok(Asn1 {
            root,
            data: self.result,
        })
    }

    /// Parse a single ASN.1 module definition
    fn module_definition(&mut self) -> Result<()> {
        let temp_start = self.temp_result.len();

        self.module_identifier()?;
        self.try_consume(&[TokenKind::KwDefinitions])?;
        self.module_defaults()?;
        self.try_consume(&[TokenKind::Assignment])?;
        self.try_consume(&[TokenKind::KwBegin])?;
        while self.peek(0).map_or(false, |t| t.value != "END") {
            self.assignment()?;
        }
        self.try_consume(&[TokenKind::KwEnd])?;

        self.end_temp_vec(temp_start, Asn1Tag::ModuleDefinition);
        Ok(())
    }

    /// Identifier at the start of a module
    fn module_identifier(&mut self) -> Result<()> {
        let temp_start = self.temp_result.len();

        self.try_consume(&[TokenKind::ModuleReference])?;

        self.end_temp_vec(temp_start, Asn1Tag::ModuleIdentifier);
        Ok(())
    }

    /// The bit between the `DEFINITIONS` keyword and the assignment
    fn module_defaults(&mut self) -> Result<()> {
        let temp_start = self.temp_result.len();

        {
            let temp_start = self.temp_result.len();
            if self.try_consume(&[TokenKind::EncodingReference]).is_ok() {
                self.try_consume(&[TokenKind::KwInstructions])?;
            }
            self.end_temp_vec(temp_start, Asn1Tag::EncodingReferenceDefault);
        }
        {
            let temp_start = self.temp_result.len();
            if self
                .try_consume(&[
                    TokenKind::KwExplicit,
                    TokenKind::KwImplicit,
                    TokenKind::KwAutomatic,
                ])
                .is_ok()
            {
                self.try_consume(&[TokenKind::KwTags])?;
            }
            self.end_temp_vec(temp_start, Asn1Tag::TagDefault);
        }
        {
            let temp_start = self.temp_result.len();
            if self.try_consume(&[TokenKind::KwExtensibility]).is_ok() {
                self.try_consume(&[TokenKind::KwImplied])?;
            }
            self.end_temp_vec(temp_start, Asn1Tag::ExtensionDefault);
        }

        self.end_temp_vec(temp_start, Asn1Tag::ModuleIdentifier);
        Ok(())
    }

    /// Parse a single assignment to a name
    fn assignment(&mut self) -> Result<()> {
        let temp_start = self.temp_result.len();

        let name = self.try_consume(&[TokenKind::TypeReference, TokenKind::ValueReference])?;

        match name.kind {
            TokenKind::TypeReference => {
                self.try_consume(&[TokenKind::Assignment])?;
                self.ty()?;
                self.end_temp_vec(temp_start, Asn1Tag::TypeAssignment)
            }
            TokenKind::ValueReference => {
                self.ty()?;
                self.try_consume(&[TokenKind::Assignment])?;
                self.value()?;
                self.end_temp_vec(temp_start, Asn1Tag::ValueAssignment)
            }
            _ => panic!("try consume error"),
        }

        Ok(())
    }

    /// Parse a type declaration
    fn ty(&mut self) -> Result<()> {
        let temp_start = self.temp_result.len();

        self.try_consume(&[TokenKind::KwBoolean])?;

        self.end_temp_vec(temp_start, Asn1Tag::Type);
        Ok(())
    }

    /// Parse a value
    fn value(&mut self) -> Result<()> {
        let temp_start = self.temp_result.len();

        self.try_consume(&[TokenKind::KwTrue, TokenKind::KwFalse])?;

        self.end_temp_vec(temp_start, Asn1Tag::Value);
        Ok(())
    }

    /// If the next token is of the provided type, consumes and returns it,
    /// otherwise returns an error.  Can check for tokens not produced by the lexer.
    /// If the token is consumed it pushes it into temp_result.
    fn try_consume(&mut self, kind: &'static [TokenKind]) -> Result<Token<'a>> {
        let tok = self.peek(0).ok_or(ParseError::Expected(kind, None))?;

        let kw = keywords().get(tok.value);

        for &kind in kind {
            match kind {
                // Identifiers cannot be passed straight through as an actual
                // identifier and the ones made by the lexer are different.
                TokenKind::Identifier | TokenKind::ValueReference
                    if is_first_lower(tok) && kw.is_none() =>
                {
                    self.next();
                    let tok = Token { kind, ..tok };
                    self.temp_result.push(TreeContent::Token(tok));
                    return Ok(tok);
                }

                // pass through for simple tokens
                _ if tok.kind == kind => {
                    self.next();
                    self.temp_result.push(TreeContent::Token(tok));
                    return Ok(tok);
                }

                _ if kw.map_or(false, |k| *k == kind) => {
                    self.next();
                    let tok = Token { kind, ..tok };
                    self.temp_result.push(TreeContent::Token(tok));
                    return Ok(tok);
                }

                // type reference = identifier with uppercase first letter
                TokenKind::TypeReference | TokenKind::ModuleReference
                    if tok.kind == TokenKind::Identifier
                        && !is_first_lower(tok)
                        && kw.is_none() =>
                {
                    self.next();
                    let tok = Token { kind, ..tok };
                    self.temp_result.push(TreeContent::Token(tok));
                    return Ok(tok);
                }

                // encoding reference = identifier with uppercase first letter
                TokenKind::EncodingReference
                    if tok.kind == TokenKind::Identifier && is_not_lower(tok) && kw.is_none() =>
                {
                    self.next();
                    let tok = Token { kind, ..tok };
                    self.temp_result.push(TreeContent::Token(tok));
                    return Ok(tok);
                }

                _ => (),
            }
        }

        Err(ParseError::Expected(kind, Some(tok.to_owned())))
    }

    /// Get the next token that is not an error or comment directly from the lexer.
    fn next(&mut self) -> Option<Token<'a>> {
        while let Some(tok) = self.tokens.pop_front() {
            match tok.kind {
                TokenKind::SingleComment
                | TokenKind::MultiComment
                | TokenKind::Unrecognised
                | TokenKind::NonTerminatedComment => {
                    self.temp_result.push(TreeContent::Token(tok));
                }
                _ => return Some(tok),
            }
        }

        None
    }

    /// Peek a token without consuming it
    fn peek(&mut self, mut n: usize) -> Option<Token<'a>> {
        for &tok in &self.tokens {
            match tok.kind {
                TokenKind::SingleComment
                | TokenKind::MultiComment
                | TokenKind::Unrecognised
                | TokenKind::NonTerminatedComment => (),
                _ if n == 0 => return Some(tok),
                _ => n -= 1,
            }
        }

        None
    }

    fn end_temp_vec(&mut self, temp_start: usize, tag: Asn1Tag) {
        let start = self.result.len();
        let count = self.temp_result.len() - temp_start;

        self.result.extend(self.temp_result.drain(temp_start..));

        self.temp_result
            .push(TreeContent::Tree { tag, start, count })
    }
}

/// Get a mapping from keyword strings to their token kind
fn keywords() -> &'static HashMap<&'static str, TokenKind> {
    static KEYWORDS: OnceLock<HashMap<&'static str, TokenKind>> = OnceLock::new();
    KEYWORDS.get_or_init(|| HashMap::from(token::KEYWORD_DATA))
}

/// Is the first character of this string a lowercase letter?
fn is_first_lower(tok: Token<'_>) -> bool {
    tok.value
        .chars()
        .next()
        .map_or(false, |c| c.is_ascii_lowercase())
}

/// Are none of the characters lower case
fn is_not_lower(tok: Token<'_>) -> bool {
    tok.value.chars().all(|c| !c.is_ascii_lowercase())
}

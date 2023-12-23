use std::{
    collections::{HashMap, VecDeque},
    sync::OnceLock,
};

use crate::{
    ast::{Asn1, Assignment, ModuleDefinition, Type, TypeAssignment, Value, ValueAssignment},
    lexer::Lexer,
    token::{self, Token, TokenBuffer, TokenKind},
};

/// Parser for ASN.1 definition files
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Parser<'a> {
    tokens: VecDeque<Token<'a>>,
    errors: Vec<ParseError>,
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
            errors: vec![],
        }
    }

    /// Run the parser to produce a set of ASN.1 definitions
    pub fn run(mut self) -> Result<Asn1> {
        let mut modules = vec![];
        while self.peek(0).is_some() {
            modules.push(self.module_definition()?);
        }

        if !self.errors.is_empty() {
            Err(ParseError::Multiple(self.errors))
        } else {
            Ok(Asn1 { modules })
        }
    }

    /// Parse a single ASN.1 module definition
    fn module_definition(&mut self) -> Result<ModuleDefinition> {
        let mut assignments = vec![];
        while self.peek(0).is_some() {
            assignments.push(self.assignment()?);
        }

        Ok(ModuleDefinition { assignments })
    }

    /// Parse a single assignment to a name
    fn assignment(&mut self) -> Result<Assignment> {
        let name = self.try_consume(&[TokenKind::TypeReference, TokenKind::ValueReference])?;

        match name.kind {
            TokenKind::TypeReference => {
                self.try_consume(&[TokenKind::Assignment])?;
                Ok(Assignment::Type(TypeAssignment {
                    type_reference: name.to_owned(),
                    ty: self.ty()?,
                }))
            }
            TokenKind::ValueReference => {
                let ty = self.ty()?;
                self.try_consume(&[TokenKind::Assignment])?;
                Ok(Assignment::Value(ValueAssignment {
                    type_reference: name.to_owned(),
                    ty,
                    value: self.value()?,
                }))
            }
            _ => panic!("try consume error"),
        }
    }

    /// Parse a type declaration
    fn ty(&mut self) -> Result<Type> {
        let tok = self.try_consume(&[TokenKind::KwBoolean])?;

        Ok(Type::Boolean)
    }

    /// Parse a value
    fn value(&mut self) -> Result<Value> {
        let value = self.try_consume(&[TokenKind::KwTrue, TokenKind::KwFalse])?;

        Ok(Value::Boolean(value.kind == TokenKind::KwTrue))
    }

    /// If the next token is of the provided type, consumes and returns it,
    /// otherwise returns an error.  Can check for tokens not produced by the lexer.
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
                    return Ok(Token { kind, ..tok });
                }

                // pass through for simple tokens
                _ if tok.kind == kind => {
                    self.next();
                    return Ok(tok);
                }

                _ if kw.map_or(false, |k| *k == kind) => {
                    self.next();
                    return Ok(Token { kind, ..tok });
                }

                // type reference = identifier with uppercase first letter
                TokenKind::TypeReference
                    if tok.kind == TokenKind::Identifier
                        && !is_first_lower(tok)
                        && kw.is_none() =>
                {
                    self.next();
                    return Ok(Token {
                        kind: TokenKind::TypeReference,
                        ..tok
                    });
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
                TokenKind::SingleComment | TokenKind::MultiComment => (),
                TokenKind::Unrecognised | TokenKind::NonTerminatedComment => {
                    self.errors.push(ParseError::Token(tok.to_owned()));
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

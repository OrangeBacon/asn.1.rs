use crate::{
    ast::{Asn1, Asn1Tag, TreeContent},
    lexer::{Lexer, LexerError, Result},
    token::{Token, TokenKind},
};

/// Parser for ASN.1 definition files
#[derive(Debug, Clone)]
pub struct Parser<'a> {
    /// Lexer to get tokens from a source file
    lexer: Lexer<'a>,

    /// The partial tree constructed by the parser
    result: Vec<TreeContent<'a>>,

    /// Temporary storage used when making the tree
    temp_result: Vec<TreeContent<'a>>,
}

impl<'a> Parser<'a> {
    /// Create a new parser from a lexer
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self {
            lexer,
            result: vec![],
            temp_result: vec![],
        }
    }

    /// Run the parser to produce a set of ASN.1 definitions
    pub fn run(mut self) -> Result<Asn1<'a>> {
        while !self.lexer.is_eof() {
            self.module_definition()?;
        }

        // handle comments at the end of the file after all meaningful tokens
        let _ = self.next(&[]);

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
        self.next(&[TokenKind::KwDefinitions])?;
        self.module_defaults()?;
        self.next(&[TokenKind::Assignment])?;
        self.next(&[TokenKind::KwBegin])?;
        while matches!(
            self.peek(&[TokenKind::KwEnd]),
            Err(LexerError::Expected { .. })
        ) {
            self.assignment()?;
        }
        self.next(&[TokenKind::KwEnd])?;

        self.end_temp_vec(temp_start, Asn1Tag::ModuleDefinition);
        Ok(())
    }

    /// Identifier at the start of a module
    fn module_identifier(&mut self) -> Result<()> {
        let temp_start = self.temp_result.len();

        self.next(&[TokenKind::ModuleReference])?;

        let definitive_start = self.temp_result.len();
        let tok = self.peek(&[TokenKind::LeftCurly, TokenKind::KwDefinitions])?;
        if tok.kind == TokenKind::LeftCurly {
            self.next(&[TokenKind::LeftCurly])?;

            let mut kind = &[TokenKind::Identifier, TokenKind::Number][..];

            loop {
                let tok = self.next(kind)?;
                if tok.kind == TokenKind::RightCurly {
                    break;
                }

                if tok.kind == TokenKind::Identifier && self.next(&[TokenKind::LeftParen]).is_ok() {
                    self.next(&[TokenKind::Number])?;
                    self.next(&[TokenKind::RightParen])?;
                }

                kind = &[
                    TokenKind::Identifier,
                    TokenKind::Number,
                    TokenKind::RightCurly,
                ];
            }
            self.end_temp_vec(definitive_start, Asn1Tag::DefinitiveOID);

            self.peek(&[TokenKind::DoubleQuote, TokenKind::KwDefinitions])?;

            let _ = self.iri_value();
        }

        self.end_temp_vec(temp_start, Asn1Tag::ModuleIdentifier);
        Ok(())
    }

    /// The bit between the `DEFINITIONS` keyword and the assignment
    fn module_defaults(&mut self) -> Result<()> {
        let temp_start = self.temp_result.len();

        {
            let temp_start = self.temp_result.len();
            self.peek(&[
                TokenKind::EncodingReference,
                TokenKind::KwExplicit,
                TokenKind::KwImplicit,
                TokenKind::KwAutomatic,
                TokenKind::KwExtensibility,
                TokenKind::Assignment,
            ])?;
            if self.next(&[TokenKind::EncodingReference]).is_ok() {
                self.next(&[TokenKind::KwInstructions])?;
            }
            self.end_temp_vec(temp_start, Asn1Tag::EncodingReferenceDefault);
        }
        {
            self.peek(&[
                TokenKind::KwExplicit,
                TokenKind::KwImplicit,
                TokenKind::KwAutomatic,
                TokenKind::KwExtensibility,
                TokenKind::Assignment,
            ])?;
            let temp_start = self.temp_result.len();
            if self
                .next(&[
                    TokenKind::KwExplicit,
                    TokenKind::KwImplicit,
                    TokenKind::KwAutomatic,
                ])
                .is_ok()
            {
                self.next(&[TokenKind::KwTags])?;
            }
            self.end_temp_vec(temp_start, Asn1Tag::TagDefault);
        }
        {
            self.peek(&[TokenKind::KwExtensibility, TokenKind::Assignment])?;
            let temp_start = self.temp_result.len();
            if self.next(&[TokenKind::KwExtensibility]).is_ok() {
                self.next(&[TokenKind::KwImplied])?;
            }
            self.end_temp_vec(temp_start, Asn1Tag::ExtensionDefault);
        }

        self.end_temp_vec(temp_start, Asn1Tag::ModuleIdentifier);
        Ok(())
    }

    /// Parse a single assignment to a name
    fn assignment(&mut self) -> Result<()> {
        let temp_start = self.temp_result.len();

        let name = self.next(&[
            TokenKind::TypeReference,
            TokenKind::ValueReference,
            TokenKind::KwEnd,
        ])?;

        match name.kind {
            TokenKind::KwEnd => {
                // shouldn't get here but oh well, end is in the list so that
                // better expected lines can be generated
                return Ok(());
            }
            TokenKind::TypeReference => {
                self.next(&[TokenKind::Assignment])?;
                self.ty()?;
                self.end_temp_vec(temp_start, Asn1Tag::TypeAssignment)
            }
            TokenKind::ValueReference => {
                self.ty()?;
                self.next(&[TokenKind::Assignment])?;
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

        self.next(&[TokenKind::KwBoolean, TokenKind::KwNull, TokenKind::KwOidIri])?;

        self.end_temp_vec(temp_start, Asn1Tag::Type);
        Ok(())
    }

    /// Parse a value
    fn value(&mut self) -> Result<()> {
        let temp_start = self.temp_result.len();

        let tok = self.peek(&[
            TokenKind::DoubleQuote,
            TokenKind::KwTrue,
            TokenKind::KwFalse,
            TokenKind::KwNull,
        ])?;
        if tok.kind == TokenKind::DoubleQuote {
            self.iri_value()?;
        } else {
            self.next(&[TokenKind::KwTrue, TokenKind::KwFalse, TokenKind::KwNull])?;
        }

        self.end_temp_vec(temp_start, Asn1Tag::Value);
        Ok(())
    }

    /// Parse an internationalised resource identifier
    fn iri_value(&mut self) -> Result<()> {
        let temp_start = self.temp_result.len();

        self.next(&[TokenKind::DoubleQuote])?;
        self.next(&[TokenKind::ForwardSlash])?;
        self.next(&[
            TokenKind::IntegerUnicodeLabel,
            TokenKind::NonIntegerUnicodeLabel,
        ])?;

        loop {
            let next = self.next(&[TokenKind::DoubleQuote, TokenKind::ForwardSlash])?;
            if next.kind == TokenKind::DoubleQuote {
                break;
            }
            self.next(&[
                TokenKind::IntegerUnicodeLabel,
                TokenKind::NonIntegerUnicodeLabel,
            ])?;
        }

        self.end_temp_vec(temp_start, Asn1Tag::IriValue);
        Ok(())
    }

    /// Get the next token that is not a comment directly from the lexer.
    fn next(&mut self, kind: &'static [TokenKind]) -> Result<Token<'a>> {
        loop {
            let tok = self.lexer.next(kind)?;
            self.temp_result.push(TreeContent::Token(tok));

            if tok.kind == TokenKind::SingleComment || tok.kind == TokenKind::MultiComment {
            } else {
                return Ok(tok);
            }
        }
    }

    /// Peek a token without consuming it
    fn peek(&mut self, kind: &'static [TokenKind]) -> Result<Token<'a>> {
        self.lexer.peek(kind)
    }

    /// Close an ast tree node with the given tag to describe the node
    fn end_temp_vec(&mut self, temp_start: usize, tag: Asn1Tag) {
        let start = self.result.len();
        let count = self.temp_result.len() - temp_start;

        self.result.extend(self.temp_result.drain(temp_start..));

        self.temp_result
            .push(TreeContent::Tree { tag, start, count })
    }
}

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
        // TODO: exports, imports

        // ensure there is at least one assignment
        self.peek(&[TokenKind::TypeReference, TokenKind::ValueReference])?;
        while matches!(
            self.peek(&[TokenKind::KwEnd]),
            Err(LexerError::Expected { .. })
        ) {
            self.assignment()?;
        }

        // TODO: encoding control sections

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

            let _ = self.iri_value(false);
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

        // TODO: Value set type assignment
        // TODO: Object class assignment
        // TODO: Object set assignment
        // TODO: Parameterized assignment

        match name.kind {
            TokenKind::KwEnd => {
                // shouldn't get here but oh well, end is in the list so that
                // better expected lines can be generated
                return Ok(());
            }
            TokenKind::TypeReference => {
                self.next(&[TokenKind::Assignment])?;
                self.ty(false)?;
                self.end_temp_vec(temp_start, Asn1Tag::TypeAssignment)
            }
            TokenKind::ValueReference => {
                let is_assign = self.ty(true)?;
                self.next(&[TokenKind::Assignment])?;

                if is_assign {
                    self.xml_typed_value()?;
                    self.end_temp_vec(temp_start, Asn1Tag::XMLAssignment)
                } else {
                    self.value()?;
                    self.end_temp_vec(temp_start, Asn1Tag::ValueAssignment)
                }
            }
            _ => panic!("try consume error"),
        }

        Ok(())
    }

    /// Parse a type declaration.  If `or_assign` is true, then could also return
    /// an assignment token, if it did, the return value is true.  In all other cases
    /// the return value is false.
    fn ty(&mut self, or_assign: bool) -> Result<bool> {
        let temp_start = self.temp_result.len();

        // TODO: Bit string, character string, choice, date, date time, duration
        // embedded pdv, enumerated, external, instance of, integer, object class field,
        // object identifier, octet string, real, relative iri, relative oid, sequence,
        // sequence of, set, set of, prefixed, time, time of day.
        // TODO: referenced type, constrained type

        let tok = self.peek(&[
            TokenKind::KwBoolean,
            TokenKind::KwNull,
            TokenKind::KwOidIri,
            TokenKind::KwInteger,
            TokenKind::Assignment,
        ])?;
        match tok.kind {
            TokenKind::Assignment if or_assign => {
                return Ok(true);
            }
            TokenKind::KwInteger => {
                self.integer_type()?;
            }
            _ => {
                self.next(&[TokenKind::KwBoolean, TokenKind::KwNull, TokenKind::KwOidIri])?;
            }
        }

        self.end_temp_vec(temp_start, Asn1Tag::Type);
        Ok(false)
    }

    /// Integer type definition, including named numbers
    fn integer_type(&mut self) -> Result<()> {
        let temp_start = self.temp_result.len();

        self.next(&[TokenKind::KwInteger])?;

        // TODO: add expected `{` after integer type to anything parsed after
        // an integer type definition, but a pain to put it inside the parser as
        // a check would have to be added after every type parse location

        if self.next(&[TokenKind::LeftCurly]).is_ok() {
            loop {
                self.next(&[TokenKind::Identifier])?;

                self.next(&[TokenKind::LeftParen])?;
                let tok = self.peek(&[
                    TokenKind::Number,
                    TokenKind::Hyphen,
                    TokenKind::ValueReference,
                    TokenKind::ModuleReference,
                ])?;
                match tok.kind {
                    TokenKind::Number => {
                        self.next(&[TokenKind::Number])?;
                    }
                    TokenKind::Hyphen => {
                        self.next(&[TokenKind::Hyphen])?;
                        self.next(&[TokenKind::Number])?;
                    }
                    _ => self.defined_value()?,
                }

                self.next(&[TokenKind::RightParen])?;
                let tok = self.next(&[TokenKind::Comma, TokenKind::RightCurly])?;

                if tok.kind == TokenKind::RightCurly {
                    break;
                }
            }
        }

        self.end_temp_vec(temp_start, Asn1Tag::IntegerType);
        Ok(())
    }

    /// Parse a value
    fn value(&mut self) -> Result<()> {
        let temp_start = self.temp_result.len();

        // TODO: bit string, character string, choice, embedded pdv, enumerated,
        // external, instance of, integer, object identifier, octet string, real
        // relative iri, relative oid, sequence, sequence of, set, set of, prefixed,
        // time
        // TODO: referenced value, object class field value

        let tok = self.peek(&[
            TokenKind::DoubleQuote,
            TokenKind::KwTrue,
            TokenKind::KwFalse,
            TokenKind::KwNull,
            TokenKind::Number,
            TokenKind::Hyphen,
            TokenKind::Identifier,
        ])?;
        match tok.kind {
            TokenKind::Number | TokenKind::Hyphen | TokenKind::Identifier => {
                self.integer_value()?
            }
            TokenKind::DoubleQuote => self.iri_value(false)?,
            _ => {
                self.next(&[TokenKind::KwTrue, TokenKind::KwFalse, TokenKind::KwNull])?;
            }
        }
        self.end_temp_vec(temp_start, Asn1Tag::Value);
        Ok(())
    }

    /// parse reference to defined value
    fn defined_value(&mut self) -> Result<()> {
        let temp_start = self.temp_result.len();

        // TODO: parameterized value

        let tok = self.peek(&[TokenKind::ValueReference, TokenKind::ModuleReference])?;
        if tok.kind == TokenKind::ModuleReference {
            self.external_value_reference()?;
        } else {
            self.next(&[TokenKind::ValueReference])?;
        }

        self.end_temp_vec(temp_start, Asn1Tag::DefinedValue);
        Ok(())
    }

    /// Parse an internationalised resource identifier
    fn iri_value(&mut self, xml: bool) -> Result<()> {
        let temp_start = self.temp_result.len();

        if !xml {
            self.next(&[TokenKind::DoubleQuote])?;
        }
        self.next(&[TokenKind::ForwardSlash])?;
        self.next(&[
            TokenKind::IntegerUnicodeLabel,
            TokenKind::NonIntegerUnicodeLabel,
        ])?;

        let kind = if xml {
            &[TokenKind::XMLEndTag, TokenKind::ForwardSlash]
        } else {
            &[TokenKind::DoubleQuote, TokenKind::ForwardSlash]
        };
        loop {
            let next = self.peek(kind)?;
            if next.kind == TokenKind::DoubleQuote || next.kind == TokenKind::XMLEndTag {
                break;
            }
            self.next(&[TokenKind::ForwardSlash])?;
            self.next(&[
                TokenKind::IntegerUnicodeLabel,
                TokenKind::NonIntegerUnicodeLabel,
            ])?;
        }

        if xml {
            self.end_temp_vec(temp_start, Asn1Tag::XMLIri);
        } else {
            self.next(&[TokenKind::DoubleQuote])?;
            self.end_temp_vec(temp_start, Asn1Tag::IriValue);
        }

        Ok(())
    }

    fn integer_value(&mut self) -> Result<()> {
        let temp_start = self.temp_result.len();

        let tok = self.next(&[TokenKind::Number, TokenKind::Hyphen, TokenKind::Identifier])?;

        if tok.kind == TokenKind::Hyphen {
            self.next(&[TokenKind::Number])?;
        }

        self.end_temp_vec(temp_start, Asn1Tag::IntegerValue);
        Ok(())
    }

    /// Parse an XML Typed value for XML value assignment
    fn xml_typed_value(&mut self) -> Result<()> {
        let temp_start = self.temp_result.len();

        self.next(&[TokenKind::Less])?;

        self.non_parameterized_type_name()?;

        let tok = self.next(&[TokenKind::Greater, TokenKind::XMLSingleTagEnd])?;

        if tok.kind == TokenKind::XMLSingleTagEnd {
            self.end_temp_vec(temp_start, Asn1Tag::XMLEmptyTag);
        } else if tok.kind == TokenKind::Greater {
            self.end_temp_vec(temp_start, Asn1Tag::XMLStartTag);

            self.xml_value()?;

            let temp_start = self.temp_result.len();
            self.next(&[TokenKind::XMLEndTag])?;
            self.non_parameterized_type_name()?;
            self.next(&[TokenKind::Greater])?;
            self.end_temp_vec(temp_start, Asn1Tag::XMLEndTag);
        }

        self.end_temp_vec(temp_start, Asn1Tag::XMLTypedValue);

        Ok(())
    }

    /// Parse a value within a typed xml value
    fn xml_value(&mut self) -> Result<()> {
        let temp_start = self.temp_result.len();

        // TODO: bit string, character string, choice, embedded pdv,
        // enumerated, external, instance of, iri, object identifier,
        // octet string, real, relative iri, relative oid, sequence, sequence of,
        // set, set of, prefixed, time
        // TODO: object class field value

        let tok = self.peek(&[
            TokenKind::XMLEndTag,
            TokenKind::Less,
            TokenKind::IdentTrue,
            TokenKind::IdentFalse,
            TokenKind::XMLBoolNumber,
            TokenKind::Number,
            TokenKind::Hyphen,
            TokenKind::Identifier,
            TokenKind::ForwardSlash,
        ])?;

        match tok.kind {
            TokenKind::XMLEndTag => {
                return Ok(());
            }
            TokenKind::Less => {
                self.next(&[TokenKind::Less])?;

                let tok = self.next(&[
                    TokenKind::IdentTrue,
                    TokenKind::IdentFalse,
                    TokenKind::Identifier,
                ])?;
                self.next(&[TokenKind::XMLSingleTagEnd])?;
                let kind = match tok.kind {
                    TokenKind::Identifier => Asn1Tag::XMLInteger,
                    _ => Asn1Tag::XMLBoolean,
                };
                self.end_temp_vec(temp_start, kind);
            }
            TokenKind::IdentTrue | TokenKind::IdentFalse | TokenKind::XMLBoolNumber => {
                self.next(&[
                    TokenKind::IdentTrue,
                    TokenKind::IdentFalse,
                    TokenKind::XMLBoolNumber,
                ])?;
                self.end_temp_vec(temp_start, Asn1Tag::XMLBoolean);
            }
            TokenKind::Hyphen => {
                self.next(&[TokenKind::Hyphen])?;
                self.next(&[TokenKind::Number])?;
                self.end_temp_vec(temp_start, Asn1Tag::XMLInteger);
            }
            TokenKind::Number | TokenKind::Identifier => {
                self.next(&[TokenKind::Number, TokenKind::Identifier])?;
                self.end_temp_vec(temp_start, Asn1Tag::XMLInteger);
            }
            TokenKind::ForwardSlash => {
                self.iri_value(true)?;
            }
            _ => (),
        }

        self.end_temp_vec(temp_start, Asn1Tag::XMLValue);

        Ok(())
    }

    /// Parse the type name that is at the start of an XML element
    fn non_parameterized_type_name(&mut self) -> Result<()> {
        let temp_start = self.temp_result.len();

        // a non-parameterized type name could be an external type reference, a
        // type reference or an xml asn1 typename.  It could also be prefixed with
        // an underscore, if it would have started with the characters "XML".
        // All xml asn1 type names are either valid type references, or keywords,
        // so the validity check can be done after parsing.  The XML asn1 type name
        // token kind allows any identifier regardless of capitalisation or whether
        // it is a keyword.  The underscore is also always accepted, even if the
        // next characters are not "XML", so should be checked later.

        let tok = self.peek(&[
            TokenKind::Underscore,
            TokenKind::TypeReference,
            TokenKind::XMLAsn1TypeName,
        ])?;

        if tok.kind == TokenKind::Underscore {
            self.next(&[TokenKind::Underscore])?;
        }

        let tok = self.next(&[TokenKind::TypeReference, TokenKind::XMLAsn1TypeName])?;
        if tok.kind == TokenKind::TypeReference {
            let tok = self.peek(&[
                TokenKind::Dot,
                TokenKind::Greater,
                TokenKind::XMLSingleTagEnd,
            ])?;
            if tok.kind == TokenKind::Dot {
                self.next(&[TokenKind::Dot])?;
                self.next(&[TokenKind::TypeReference])?;
            }
        }

        self.end_temp_vec(temp_start, Asn1Tag::XMLTypedValue);

        Ok(())
    }

    /// Parse a reference to an external value
    fn external_value_reference(&mut self) -> Result<()> {
        let temp_start = self.temp_result.len();

        self.next(&[TokenKind::ModuleReference])?;
        self.next(&[TokenKind::Dot])?;
        self.next(&[TokenKind::ValueReference])?;

        self.end_temp_vec(temp_start, Asn1Tag::ExternalTypeReference);

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

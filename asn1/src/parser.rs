use crate::{
    cst::{Asn1, Asn1Tag, TreeContent},
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

    /// data to finish constructing a partial cst in error cases
    error_nodes: Vec<TempVec>,
}

/// The kind of tokens accepted along side a type
enum TypeStartKind {
    /// nothing extra
    None,

    /// Also allow an assignment token `::=`
    Assignment,

    /// Also allow a signed number or a defined value for the exception spec
    Exception,
}

/// Helper for constructing cst tree nodes from the temp_result array in the error
/// case, when unwinding (through result) through the parser.
#[derive(Debug, Clone)]
struct TempVec {
    tag: Asn1Tag,
    offset: usize,
}

impl<'a> Parser<'a> {
    /// Create a new parser from a lexer
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self {
            lexer,
            result: vec![],
            temp_result: vec![],
            error_nodes: vec![],
        }
    }

    /// Run the parser to produce a set of ASN.1 definitions
    pub fn run(mut self) -> Result<Asn1<'a>> {
        self.start_temp_vec(Asn1Tag::Root);

        while !self.lexer.is_eof() {
            self.module_definition()?;
        }

        // handle comments at the end of the file after all meaningful tokens
        let _ = self.next(&[]);

        self.end_temp_vec(Asn1Tag::Root);
        let root = self.result.len();
        self.result.push(self.temp_result[0]);

        println!("{:?}", self.error_nodes);

        Ok(Asn1 {
            root,
            data: self.result,
        })
    }

    /// Parse a single ASN.1 module definition
    fn module_definition(&mut self) -> Result<()> {
        self.start_temp_vec(Asn1Tag::ModuleDefinition);

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

        self.end_temp_vec(Asn1Tag::ModuleDefinition);
        Ok(())
    }

    /// Identifier at the start of a module
    fn module_identifier(&mut self) -> Result<()> {
        self.start_temp_vec(Asn1Tag::ModuleIdentifier);

        self.next(&[TokenKind::ModuleReference])?;

        let tok = self.peek(&[TokenKind::LeftCurly, TokenKind::KwDefinitions])?;
        if tok.kind == TokenKind::LeftCurly {
            self.start_temp_vec(Asn1Tag::DefinitiveOID);
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
            self.end_temp_vec(Asn1Tag::DefinitiveOID);

            self.peek(&[TokenKind::DoubleQuote, TokenKind::KwDefinitions])?;

            let _ = self.iri_value(false);
        }

        self.end_temp_vec(Asn1Tag::ModuleIdentifier);
        Ok(())
    }

    /// The bit between the `DEFINITIONS` keyword and the assignment
    fn module_defaults(&mut self) -> Result<()> {
        self.start_temp_vec(Asn1Tag::ModuleDefaults);

        {
            self.start_temp_vec(Asn1Tag::EncodingReferenceDefault);
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
            self.end_temp_vec(Asn1Tag::EncodingReferenceDefault);
        }
        {
            self.start_temp_vec(Asn1Tag::TagDefault);
            self.peek(&[
                TokenKind::KwExplicit,
                TokenKind::KwImplicit,
                TokenKind::KwAutomatic,
                TokenKind::KwExtensibility,
                TokenKind::Assignment,
            ])?;
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
            self.end_temp_vec(Asn1Tag::TagDefault);
        }
        {
            self.start_temp_vec(Asn1Tag::ExtensionDefault);
            self.peek(&[TokenKind::KwExtensibility, TokenKind::Assignment])?;
            if self.next(&[TokenKind::KwExtensibility]).is_ok() {
                self.next(&[TokenKind::KwImplied])?;
            }
            self.end_temp_vec(Asn1Tag::ExtensionDefault);
        }

        self.end_temp_vec(Asn1Tag::ModuleDefaults);
        Ok(())
    }

    /// Parse a single assignment to a name
    fn assignment(&mut self) -> Result<()> {
        self.start_temp_vec(Asn1Tag::Assignment);

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
                self.start_temp_vec(Asn1Tag::TypeAssignment);
                self.next(&[TokenKind::Assignment])?;
                self.ty(TypeStartKind::None)?;
                self.end_temp_vec(Asn1Tag::TypeAssignment);
            }
            TokenKind::ValueReference => {
                self.start_temp_vec(Asn1Tag::ValueAssignment);

                let is_assign = self.ty(TypeStartKind::Assignment)?;
                self.next(&[TokenKind::Assignment])?;

                if is_assign {
                    self.xml_typed_value()?;
                } else {
                    self.value()?;
                }
                self.end_temp_vec(Asn1Tag::ValueAssignment)
            }
            _ => panic!("try consume error"),
        }

        self.end_temp_vec(Asn1Tag::Assignment);

        Ok(())
    }

    /// Parse a type declaration.  `kind` represents the other kinds of token that
    /// could be peeked at the start of the type definition, for error reporting
    /// purposes.  If one of them is matched, then returns true, otherwise false.
    fn ty(&mut self, kind: TypeStartKind) -> Result<bool> {
        // TODO: Bit string, character string, choice, date, date time, duration
        // embedded pdv, external, instance of, integer, object class field,
        // object identifier, octet string, real, relative iri, relative oid, sequence,
        // sequence of, set, set of, prefixed, time, time of day.
        // TODO: referenced type, constrained type

        macro_rules! kinds {
            ($($extra:path),* $(,)?) => {
                &[
                    TokenKind::KwBoolean,
                    TokenKind::KwNull,
                    TokenKind::KwOidIri,
                    TokenKind::KwInteger,
                    TokenKind::KwEnumerated,
                    $($extra),*
                ][..]
            };
        }

        let kind = match kind {
            TypeStartKind::None => kinds!(),
            TypeStartKind::Assignment => kinds!(TokenKind::Assignment),
            TypeStartKind::Exception => kinds!(TokenKind::Hyphen, TokenKind::Number),
        };
        let tok = self.peek(kind)?;

        match tok.kind {
            TokenKind::Assignment | TokenKind::Hyphen | TokenKind::Number => {
                return Ok(true);
            }
            TokenKind::KwInteger => {
                self.start_temp_vec(Asn1Tag::Type);
                self.integer_type()?
            }
            TokenKind::KwEnumerated => {
                self.start_temp_vec(Asn1Tag::Type);
                self.enumerated_type()?
            }
            _ => {
                self.start_temp_vec(Asn1Tag::Type);
                self.next(&[TokenKind::KwBoolean, TokenKind::KwNull, TokenKind::KwOidIri])?;
            }
        }

        self.end_temp_vec(Asn1Tag::Type);
        Ok(false)
    }

    /// Integer type definition, including named numbers
    fn integer_type(&mut self) -> Result<()> {
        self.start_temp_vec(Asn1Tag::IntegerType);

        self.next(&[TokenKind::KwInteger])?;

        // TODO: add expected `{` after integer type to anything parsed after
        // an integer type definition, but a pain to put it inside the parser as
        // a check would have to be added after every type parse location

        if self.next(&[TokenKind::LeftCurly]).is_ok() {
            loop {
                self.named_number(false)?;

                let tok = self.next(&[TokenKind::Comma, TokenKind::RightCurly])?;

                if tok.kind == TokenKind::RightCurly {
                    break;
                }
            }
        }

        self.end_temp_vec(Asn1Tag::IntegerType);
        Ok(())
    }

    /// Parse an enum type declaration
    fn enumerated_type(&mut self) -> Result<()> {
        self.start_temp_vec(Asn1Tag::EnumeratedType);

        self.next(&[TokenKind::KwEnumerated])?;
        self.next(&[TokenKind::LeftCurly])?;

        self.enum_item_list(true)?;
        let tok = self.peek(&[TokenKind::RightCurly, TokenKind::Ellipsis])?;
        if tok.kind == TokenKind::Ellipsis {
            self.next(&[TokenKind::Ellipsis])?;

            let kind = if self.exception_spec()? {
                &[TokenKind::Comma, TokenKind::RightCurly][..]
            } else {
                &[
                    TokenKind::Comma,
                    TokenKind::RightCurly,
                    TokenKind::Exclamation,
                ]
            };

            let tok = self.peek(kind)?;
            if tok.kind == TokenKind::Comma {
                self.next(&[TokenKind::Comma])?;
                self.enum_item_list(false)?;
            }
        }

        self.next(&[TokenKind::RightCurly])?;

        self.end_temp_vec(Asn1Tag::EnumeratedType);
        Ok(())
    }

    /// The list of items on the inside of an enum, is item "Enumeration" in the
    /// specification.  If first is true will also break out of the loop if there
    /// is an ellipsis, not just a curly brace.
    fn enum_item_list(&mut self, first: bool) -> Result<()> {
        self.start_temp_vec(Asn1Tag::EnumItemList);

        loop {
            self.named_number(true)?;

            let tok = self.peek(&[TokenKind::RightCurly, TokenKind::Comma])?;
            if tok.kind == TokenKind::RightCurly {
                break;
            }
            self.next(&[TokenKind::Comma])?;

            if first {
                let tok = self.peek(&[TokenKind::Identifier, TokenKind::Ellipsis])?;
                if tok.kind == TokenKind::Ellipsis {
                    break;
                }
            }
        }

        self.end_temp_vec(Asn1Tag::EnumItemList);
        Ok(())
    }

    /// An integer value with a name, `hello(5)`.  If `is_enum`, then it will
    /// also allow returning just an identifier, without the parenthesized
    /// value.
    fn named_number(&mut self, is_enum: bool) -> Result<()> {
        let tag = if is_enum {
            Asn1Tag::EnumItem
        } else {
            Asn1Tag::NamedNumber
        };
        self.start_temp_vec(tag);

        self.next(&[TokenKind::Identifier])?;

        let kind = if is_enum {
            &[
                TokenKind::LeftParen,
                TokenKind::Comma,
                TokenKind::RightCurly,
            ][..]
        } else {
            &[TokenKind::LeftParen]
        };

        let tok = self.peek(kind)?;
        if tok.kind != TokenKind::LeftParen {
            self.end_temp_vec(tag);
            return Ok(());
        }
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

        self.end_temp_vec(tag);
        Ok(())
    }

    /// Parse a specifier that there is an un-specified constraint in the asn.1
    /// file.  returns true if the exception spec was not empty.
    fn exception_spec(&mut self) -> Result<bool> {
        self.start_temp_vec(Asn1Tag::ExceptionSpec);

        if self.next(&[TokenKind::Exclamation]).is_err() {
            self.end_temp_vec(Asn1Tag::ExceptionSpec);
            return Ok(false);
        }

        // TODO: Defined value option
        // = external value reference
        // | value reference

        if self.ty(TypeStartKind::Exception)? {
            let tok = self.next(&[TokenKind::Hyphen, TokenKind::Number])?;
            if tok.kind == TokenKind::Hyphen {
                self.next(&[TokenKind::Number])?;
            }
        } else {
            self.next(&[TokenKind::Colon])?;
            self.value()?;
        }

        self.end_temp_vec(Asn1Tag::ExceptionSpec);
        Ok(true)
    }

    /// Parse a value
    fn value(&mut self) -> Result<()> {
        self.start_temp_vec(Asn1Tag::Value);

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
        self.end_temp_vec(Asn1Tag::Value);
        Ok(())
    }

    /// parse reference to defined value
    fn defined_value(&mut self) -> Result<()> {
        self.start_temp_vec(Asn1Tag::DefinedValue);

        // TODO: parameterized value

        let tok = self.peek(&[TokenKind::ValueReference, TokenKind::ModuleReference])?;
        if tok.kind == TokenKind::ModuleReference {
            self.external_value_reference()?;
        } else {
            self.next(&[TokenKind::ValueReference])?;
        }

        self.end_temp_vec(Asn1Tag::DefinedValue);
        Ok(())
    }

    /// Parse an internationalised resource identifier
    fn iri_value(&mut self, xml: bool) -> Result<()> {
        let tag = if xml {
            Asn1Tag::XMLIri
        } else {
            Asn1Tag::IriValue
        };
        self.start_temp_vec(tag);

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

        if !xml {
            self.next(&[TokenKind::DoubleQuote])?;
        }

        self.end_temp_vec(tag);

        Ok(())
    }

    fn integer_value(&mut self) -> Result<()> {
        self.start_temp_vec(Asn1Tag::IntegerValue);

        let tok = self.next(&[TokenKind::Number, TokenKind::Hyphen, TokenKind::Identifier])?;

        if tok.kind == TokenKind::Hyphen {
            self.next(&[TokenKind::Number])?;
        }

        self.end_temp_vec(Asn1Tag::IntegerValue);
        Ok(())
    }

    /// Parse an XML Typed value for XML value assignment
    fn xml_typed_value(&mut self) -> Result<()> {
        self.start_temp_vec(Asn1Tag::XMLTypedValue);
        self.start_temp_vec(Asn1Tag::XMLTag);
        self.next(&[TokenKind::Less])?;

        self.non_parameterized_type_name()?;

        let tok = self.next(&[TokenKind::Greater, TokenKind::XMLSingleTagEnd])?;

        self.end_temp_vec(Asn1Tag::XMLTag);
        if tok.kind == TokenKind::Greater {
            self.xml_value()?;

            self.start_temp_vec(Asn1Tag::XMLTag);
            self.next(&[TokenKind::XMLEndTag])?;
            self.non_parameterized_type_name()?;
            self.next(&[TokenKind::Greater])?;
            self.end_temp_vec(Asn1Tag::XMLTag);
        }

        self.end_temp_vec(Asn1Tag::XMLTypedValue);

        Ok(())
    }

    /// Parse a value within a typed xml value
    fn xml_value(&mut self) -> Result<()> {
        self.start_temp_vec(Asn1Tag::XMLValue);

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
                self.end_temp_vec(Asn1Tag::XMLValue);
                return Ok(());
            }
            TokenKind::Less => {
                self.start_temp_vec(Asn1Tag::XMLTag);
                self.next(&[TokenKind::Less])?;

                let kind = &[
                    TokenKind::IdentTrue,
                    TokenKind::IdentFalse,
                    TokenKind::Identifier,
                ];
                let tok = self.peek(kind)?;
                let tag = if tok.kind == TokenKind::Identifier {
                    Asn1Tag::IntegerValue
                } else {
                    Asn1Tag::XMLBoolean
                };
                self.start_temp_vec(tag);

                self.next(kind)?;
                self.end_temp_vec(tag);

                self.next(&[TokenKind::XMLSingleTagEnd])?;
                self.end_temp_vec(Asn1Tag::XMLTag);
            }
            TokenKind::IdentTrue | TokenKind::IdentFalse | TokenKind::XMLBoolNumber => {
                self.start_temp_vec(Asn1Tag::XMLBoolean);
                self.next(&[
                    TokenKind::IdentTrue,
                    TokenKind::IdentFalse,
                    TokenKind::XMLBoolNumber,
                ])?;
                self.end_temp_vec(Asn1Tag::XMLBoolean);
            }
            TokenKind::Hyphen => {
                self.start_temp_vec(Asn1Tag::XMLInteger);
                self.next(&[TokenKind::Hyphen])?;
                self.next(&[TokenKind::Number])?;
                self.end_temp_vec(Asn1Tag::XMLInteger);
            }
            TokenKind::Number | TokenKind::Identifier => {
                self.start_temp_vec(Asn1Tag::XMLInteger);
                self.next(&[TokenKind::Number, TokenKind::Identifier])?;
                self.end_temp_vec(Asn1Tag::XMLInteger);
            }
            TokenKind::ForwardSlash => {
                self.iri_value(true)?;
            }
            _ => (),
        }

        self.end_temp_vec(Asn1Tag::XMLValue);

        Ok(())
    }

    /// Parse the type name that is at the start of an XML element
    fn non_parameterized_type_name(&mut self) -> Result<()> {
        self.start_temp_vec(Asn1Tag::XMLTypedValue);

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

        self.end_temp_vec(Asn1Tag::XMLTypedValue);
        Ok(())
    }

    /// Parse a reference to an external value
    fn external_value_reference(&mut self) -> Result<()> {
        self.start_temp_vec(Asn1Tag::ExternalValueReference);

        self.next(&[TokenKind::ModuleReference])?;
        self.next(&[TokenKind::Dot])?;
        self.next(&[TokenKind::ValueReference])?;

        self.end_temp_vec(Asn1Tag::ExternalValueReference);
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

    /// Start an ast tree node with the given tag to describe the node
    fn start_temp_vec(&mut self, tag: Asn1Tag) {
        self.error_nodes.push(TempVec {
            tag,
            offset: self.temp_result.len(),
        })
    }

    /// End the most recent temporary vec.
    #[track_caller]
    fn end_temp_vec(&mut self, tag: Asn1Tag) {
        let end = self.error_nodes.pop().unwrap();

        debug_assert_eq!(tag, end.tag);

        let temp_start = end.offset;

        let start = self.result.len();
        let count = self.temp_result.len() - temp_start;

        self.result.extend(self.temp_result.drain(temp_start..));

        self.temp_result.push(TreeContent::Tree {
            tag: end.tag,
            start,
            count,
        })
    }
}

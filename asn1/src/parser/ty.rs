use crate::{cst::Asn1Tag, token::TokenKind};

use super::{Parser, Result};

impl<'a> Parser<'a> {
    /// Parse a type declaration.  `kind` represents the other kinds of token that
    /// could be peeked at the start of the type definition, for error reporting
    /// purposes.  If one of them is matched, then returns true, otherwise false.
    pub(in crate::parser) fn ty(&mut self) -> Result {
        // TODO: Bit string, character string, choice, date, date time, duration
        // embedded pdv, external, instance of, integer, object class field,
        // octet string, real, relative iri, relative oid, sequence,
        // sequence of, set, set of, prefixed, time, time of day.
        // TODO: type from object,
        // value set from objects
        // TODO: constrained type

        // module ref . type ref
        // type ref
        // type { ... } // parameterized type

        let tok = self.peek(&[
            // builtin types
            TokenKind::KwBoolean,
            TokenKind::KwNull,
            TokenKind::KwOidIri,
            TokenKind::KwInteger,
            TokenKind::KwEnumerated,
            TokenKind::KwObject,
            // reference type
            TokenKind::TypeOrModuleRef,
            TokenKind::ValueRefOrIdent,
            // useful type
            TokenKind::KwGeneralizedTime,
            TokenKind::KwUTCTime,
            TokenKind::KwObjectDescriptor,
        ])?;

        self.start_temp_vec(Asn1Tag::Type);
        match tok.kind {
            TokenKind::KwInteger => self.integer_type()?,
            TokenKind::KwEnumerated => self.enumerated_type()?,
            TokenKind::KwObject => self.object_identifier_type()?,
            TokenKind::TypeOrModuleRef => self.defined_type()?,
            TokenKind::ValueRefOrIdent => self.selection_type()?,
            _ => {
                self.next(&[
                    TokenKind::KwBoolean,
                    TokenKind::KwNull,
                    TokenKind::KwOidIri,
                    TokenKind::KwGeneralizedTime,
                    TokenKind::KwUTCTime,
                    TokenKind::KwObjectDescriptor,
                ])?;
            }
        }

        self.end_temp_vec(Asn1Tag::Type);
        Ok(())
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
                let tok = self.peek(&[TokenKind::ValueRefOrIdent, TokenKind::Ellipsis])?;
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

        self.next(&[TokenKind::ValueRefOrIdent])?;

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
            TokenKind::ValueRefOrIdent,
            TokenKind::TypeOrModuleRef,
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

        if self.peek(&[TokenKind::Hyphen, TokenKind::Number]).is_ok() {
            let tok = self.next(&[TokenKind::Hyphen, TokenKind::Number])?;
            if tok.kind == TokenKind::Hyphen {
                self.next(&[TokenKind::Number])?;
            }
        } else {
            self.ty()?;
            self.next(&[TokenKind::Colon])?;
            self.value()?;
        }

        self.end_temp_vec(Asn1Tag::ExceptionSpec);
        Ok(true)
    }

    fn object_identifier_type(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::ObjectIDType);

        self.next(&[TokenKind::KwObject])?;
        self.next(&[TokenKind::KwIdentifier])?;

        self.end_temp_vec(Asn1Tag::ObjectIDType);
        Ok(())
    }

    /// Parse a reference to a previously defined type.
    /// ```bnf
    /// DefinedType ::=
    ///     ExternalTypeReference
    ///   | type_reference
    ///   | ParameterizedType
    ///   | ParameterizedValueSetType
    /// ```
    fn defined_type(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::DefinedType);

        self.next(&[TokenKind::TypeOrModuleRef])?;
        if self.peek(&[TokenKind::Dot]).is_ok() {
            self.next(&[TokenKind::Dot])?;
            self.next(&[TokenKind::TypeOrModuleRef])?;
        }

        if self.peek(&[TokenKind::LeftCurly]).is_ok() {
            self.actual_parameter_list()?;
        }

        self.end_temp_vec(Asn1Tag::DefinedType);
        Ok(())
    }

    /// Parse a selection type
    /// ```bnf
    /// SelectionType ::= identifier "<" Type
    /// ```
    fn selection_type(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::SelectionType);

        self.next(&[TokenKind::ValueRefOrIdent])?;
        self.next(&[TokenKind::Less])?;
        self.ty()?;

        self.end_temp_vec(Asn1Tag::SelectionType);
        Ok(())
    }
}

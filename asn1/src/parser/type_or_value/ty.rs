use crate::{cst::Asn1Tag, token::TokenKind};

use super::{Parser, Result, TypeOrValue, TypeOrValueResult};

impl<'a> Parser<'a> {
    /// Integer type definition, including named numbers
    pub(super) fn integer_type(&mut self, subsequent: &[TokenKind]) -> Result {
        self.start_temp_vec(Asn1Tag::IntegerType)?;

        self.next(&[TokenKind::KwInteger])?;

        let mut kind = subsequent.to_vec();
        kind.push(TokenKind::LeftCurly);

        let tok = self.peek(kind)?;
        if tok.kind == TokenKind::LeftCurly {
            self.next(&[TokenKind::LeftCurly])?;

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
    pub(super) fn enumerated_type(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::EnumeratedType)?;

        self.next(&[TokenKind::KwEnumerated])?;
        self.next(&[TokenKind::LeftCurly])?;

        self.enum_item_list(true)?;
        let tok = self.peek(&[TokenKind::RightCurly, TokenKind::Ellipsis])?;
        if tok.kind == TokenKind::Ellipsis {
            self.next(&[TokenKind::Ellipsis])?;

            let spec = self.exception_spec(&[
                TokenKind::Comma,
                TokenKind::RightCurly,
                TokenKind::Exclamation,
            ])?;

            let kind = if spec {
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
    fn enum_item_list(&mut self, first: bool) -> Result {
        self.start_temp_vec(Asn1Tag::EnumItemList)?;

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
    fn named_number(&mut self, is_enum: bool) -> Result {
        let tag = if is_enum {
            Asn1Tag::EnumItem
        } else {
            Asn1Tag::NamedNumber
        };
        self.start_temp_vec(tag)?;

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
    /// file.  Returns true if the a non-empty exception spec was parsed.
    fn exception_spec(&mut self, subsequent: &[TokenKind]) -> Result<bool> {
        self.start_temp_vec(Asn1Tag::ExceptionSpec)?;

        let mut kind = subsequent.to_vec();
        kind.push(TokenKind::Exclamation);

        let tok = self.peek(kind)?;
        if tok.kind != TokenKind::Exclamation {
            self.end_temp_vec(Asn1Tag::ExceptionSpec);
            return Ok(false);
        }
        self.next(&[TokenKind::Exclamation])?;

        // TODO: Defined value option
        // = external value reference
        // | value reference

        let res = self.type_or_value(TypeOrValue {
            is_type: true,
            is_value: false,
            alternative: &[TokenKind::Hyphen, TokenKind::Number],
            subsequent: &[TokenKind::Colon],
        })?;

        if matches!(res, TypeOrValueResult::Alternate(_)) {
            let tok = self.next(&[TokenKind::Hyphen, TokenKind::Number])?;
            if tok.kind == TokenKind::Hyphen {
                self.next(&[TokenKind::Number])?;
            }
        } else {
            self.next(&[TokenKind::Colon])?;
            self.type_or_value(TypeOrValue {
                is_type: false,
                is_value: true,
                alternative: &[],
                subsequent,
            })?;
        }

        self.end_temp_vec(Asn1Tag::ExceptionSpec);
        Ok(true)
    }

    pub(super) fn object_identifier_type(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::ObjectIDType)?;

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
    pub(super) fn defined_type(&mut self, subsequent: &[TokenKind]) -> Result {
        self.start_temp_vec(Asn1Tag::DefinedType)?;

        let mut kind = subsequent.to_vec();
        kind.push(TokenKind::Dot);

        self.next(&[TokenKind::TypeOrModuleRef])?;
        let tok = self.peek(kind)?;
        if tok.kind == TokenKind::Dot {
            self.next(&[TokenKind::Dot])?;
            self.next(&[TokenKind::TypeOrModuleRef])?;
        }

        let mut kind = subsequent.to_vec();
        kind.push(TokenKind::LeftCurly);

        let tok = self.peek(kind)?;
        if tok.kind == TokenKind::LeftCurly {
            self.actual_parameter_list()?;
        }

        self.end_temp_vec(Asn1Tag::DefinedType);
        Ok(())
    }

    /// Parse a selection type
    /// ```bnf
    /// SelectionType ::= identifier "<" Type
    /// ```
    pub(super) fn selection_type(&mut self, subsequent: &[TokenKind]) -> Result {
        self.start_temp_vec(Asn1Tag::SelectionType)?;

        self.next(&[TokenKind::ValueRefOrIdent])?;
        self.next(&[TokenKind::Less])?;
        self.type_or_value(TypeOrValue {
            is_type: true,
            is_value: false,
            alternative: &[],
            subsequent,
        })?;

        self.end_temp_vec(Asn1Tag::SelectionType);
        Ok(())
    }
}
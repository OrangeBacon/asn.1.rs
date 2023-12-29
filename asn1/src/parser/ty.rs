use crate::{cst::Asn1Tag, token::TokenKind};

use super::{Parser, Result};

/// The kind of tokens accepted along side a type
pub(in crate::parser) enum TypeStartKind {
    /// nothing extra
    None,

    /// Also allow an assignment token `::=`
    Assignment,

    /// Also allow a signed number or a defined value for the exception spec
    Exception,
}

impl<'a> Parser<'a> {
    /// Parse a type declaration.  `kind` represents the other kinds of token that
    /// could be peeked at the start of the type definition, for error reporting
    /// purposes.  If one of them is matched, then returns true, otherwise false.
    pub(in crate::parser) fn ty(&mut self, kind: TypeStartKind) -> Result<bool> {
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
}

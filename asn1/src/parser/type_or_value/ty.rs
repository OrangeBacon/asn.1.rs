use crate::{cst::Asn1Tag, token::TokenKind};

use super::{Parser, Result, TypeOrValue, TypeOrValueResult};

impl<'a> Parser<'a> {
    /// Integer type definition, including named numbers
    pub(super) fn integer_type(&mut self, expecting: TypeOrValue) -> Result {
        self.start_temp_vec(Asn1Tag::IntegerType)?;

        self.next(&[TokenKind::KwInteger])?;

        let mut kind = expecting.subsequent.to_vec();
        kind.push(TokenKind::LeftCurly);
        kind.push(TokenKind::Colon);

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

        self.open_type_field_value(expecting)?;
        Ok(())
    }

    /// Parse an enum type declaration
    pub(super) fn enumerated_type(&mut self, expecting: TypeOrValue) -> Result {
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

        self.open_type_field_value(expecting)?;

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

        self.type_or_value(TypeOrValue {
            alternative: &[],
            subsequent: &[TokenKind::RightParen],
        })?;

        self.next(&[TokenKind::RightParen])?;

        self.end_temp_vec(tag);
        Ok(())
    }

    /// Parse a specifier that there is an un-specified constraint in the asn.1
    /// file.  Returns true if the a non-empty exception spec was parsed.  All
    /// elements of the exception spec count as types or values, so check for the
    /// valid ones later.
    pub(super) fn exception_spec(&mut self, subsequent: &[TokenKind]) -> Result<bool> {
        self.start_temp_vec(Asn1Tag::ExceptionSpec)?;

        let mut kind = subsequent.to_vec();
        kind.push(TokenKind::Exclamation);

        let tok = self.peek(kind)?;
        if tok.kind != TokenKind::Exclamation {
            self.end_temp_vec(Asn1Tag::ExceptionSpec);
            return Ok(false);
        }
        self.next(&[TokenKind::Exclamation])?;

        self.type_or_value(TypeOrValue {
            subsequent,
            alternative: &[],
        })?;

        self.end_temp_vec(Asn1Tag::ExceptionSpec);
        Ok(true)
    }

    /// Parse `OBJECT IDENTIFIER` keywords
    pub(super) fn object_identifier_type(&mut self, expecting: TypeOrValue) -> Result {
        self.start_temp_vec(Asn1Tag::ObjectIDType)?;

        self.next(&[TokenKind::KwObject])?;
        self.next(&[TokenKind::KwIdentifier])?;

        self.end_temp_vec(Asn1Tag::ObjectIDType);

        self.open_type_field_value(expecting)?;
        Ok(())
    }

    /// Parse a selection type.  Assumes that the identifier has already been
    /// consumed earlier in the parser.
    /// ```bnf
    /// SelectionType ::= identifier "<" Type
    /// ```
    pub(super) fn selection_type(&mut self, subsequent: &[TokenKind]) -> Result {
        self.start_temp_vec(Asn1Tag::SelectionType)?;

        self.next(&[TokenKind::Less])?;
        self.type_or_value(TypeOrValue {
            subsequent,
            alternative: &[],
        })?;

        self.end_temp_vec(Asn1Tag::SelectionType);
        Ok(())
    }

    /// Bit string type definition
    pub(super) fn bit_string_type(&mut self, expecting: TypeOrValue) -> Result {
        self.start_temp_vec(Asn1Tag::BitStringType)?;

        self.next(&[TokenKind::KwBit])?;
        self.next(&[TokenKind::KwString])?;

        let mut kind = expecting.subsequent.to_vec();
        kind.push(TokenKind::LeftCurly);
        kind.push(TokenKind::Colon);

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

        self.end_temp_vec(Asn1Tag::BitStringType);

        self.open_type_field_value(expecting)?;
        Ok(())
    }

    /// Parse `OCTET STRING` keywords
    pub(super) fn octet_string_type(&mut self, expecting: TypeOrValue) -> Result {
        self.start_temp_vec(Asn1Tag::OctetStringType)?;

        self.next(&[TokenKind::KwOctet])?;
        self.next(&[TokenKind::KwString])?;

        self.end_temp_vec(Asn1Tag::OctetStringType);

        self.open_type_field_value(expecting)?;
        Ok(())
    }

    /// Parse `CHARACTER STRING` keywords
    pub(super) fn character_string_type(&mut self, expecting: TypeOrValue) -> Result {
        self.start_temp_vec(Asn1Tag::OctetStringType)?;

        self.next(&[TokenKind::KwCharacter])?;
        self.next(&[TokenKind::KwString])?;

        self.end_temp_vec(Asn1Tag::OctetStringType);

        self.open_type_field_value(expecting)?;
        Ok(())
    }

    /// Parse `ABSTRACT-SYNTAX` or `TYPE-IDENTIFIER` keywords and field names
    pub(super) fn object_fields(&mut self, expecting: TypeOrValue) -> Result {
        self.start_temp_vec(Asn1Tag::ObjectFields)?;

        self.next(&[TokenKind::KwAbstractSyntax, TokenKind::KwTypeIdentifier])?;

        let mut kind = expecting.subsequent.to_vec();
        kind.push(TokenKind::Dot);
        kind.push(TokenKind::Colon);
        let tok = self.peek(kind)?;
        if tok.kind == TokenKind::Dot {
            let mut kind = expecting.subsequent.to_vec();
            kind.push(TokenKind::Colon);
            self.field(&kind)?;
        }

        self.end_temp_vec(Asn1Tag::ObjectFields);

        self.open_type_field_value(expecting)?;
        Ok(())
    }

    /// Parse the `INSTANCE OF DefinedObject` type
    pub(super) fn instance_of_type(&mut self, expecting: TypeOrValue) -> Result {
        self.start_temp_vec(Asn1Tag::InstanceOfType)?;

        self.next(&[TokenKind::KwInstance])?;
        self.next(&[TokenKind::KwOf])?;

        let mut kind = expecting.subsequent.to_vec();
        kind.push(TokenKind::Colon);
        self.type_or_value(TypeOrValue {
            alternative: &[],
            subsequent: &kind,
        })?;

        self.end_temp_vec(Asn1Tag::InstanceOfType);

        self.open_type_field_value(expecting)?;
        Ok(())
    }

    /// Parse `EMBEDDED PDV` keywords
    pub(super) fn embedded_pdv_type(&mut self, expecting: TypeOrValue) -> Result {
        self.start_temp_vec(Asn1Tag::EmbeddedPDVType)?;

        self.next(&[TokenKind::KwEmbedded])?;
        self.next(&[TokenKind::KwPDV])?;

        self.end_temp_vec(Asn1Tag::EmbeddedPDVType);

        self.open_type_field_value(expecting)?;
        Ok(())
    }

    /// Parse a prefixed type
    /// `"[" encoding* "]" (IMPLICIT|EXPLICIT)? Type`.  Does not interpret the
    /// inside of the square brackets and just skips to the next closing one as
    /// neither `[` nor `]` can appear inside.
    pub(super) fn prefix_type(&mut self, expecting: TypeOrValue) -> Result {
        self.start_temp_vec(Asn1Tag::PrefixType)?;

        self.next(&[TokenKind::LeftSquare])?;
        loop {
            let tok = self.next(&[])?;
            if tok.kind == TokenKind::RightSquare {
                break;
            }
        }

        let ty = self.type_or_value(TypeOrValue {
            alternative: &[TokenKind::KwExplicit, TokenKind::KwImplicit],
            subsequent: expecting.subsequent,
        })?;
        if matches!(ty, TypeOrValueResult::Alternate(_)) {
            self.next(&[TokenKind::KwExplicit, TokenKind::KwImplicit])?;
            self.type_or_value(TypeOrValue {
                alternative: &[],
                subsequent: expecting.subsequent,
            })?;
        }

        self.end_temp_vec(Asn1Tag::PrefixType);

        self.open_type_field_value(expecting)?;
        Ok(())
    }
}

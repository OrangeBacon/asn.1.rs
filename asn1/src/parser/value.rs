use crate::{cst::Asn1Tag, token::TokenKind};

use super::{Parser, Result};

impl<'a> Parser<'a> {
    /// Parse a value
    pub(in crate::parser) fn value(&mut self) -> Result<()> {
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
    pub(in crate::parser) fn defined_value(&mut self) -> Result<()> {
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
    pub(in crate::parser) fn iri_value(&mut self, xml: bool) -> Result<()> {
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
}

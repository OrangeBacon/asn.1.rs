/*use crate::{cst::Asn1Tag, token::TokenKind};

use super::{Parser, Result};

impl<'a> Parser<'a> {
    /// Parse an XML Typed value for XML value assignment
    pub(in crate::parser) fn xml_typed_value(&mut self) -> Result<()> {
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
            TokenKind::ValueRefOrIdent,
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
                    TokenKind::ValueRefOrIdent,
                ];
                let tok = self.peek(kind)?;
                let tag = if tok.kind == TokenKind::ValueRefOrIdent {
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
            TokenKind::Number | TokenKind::ValueRefOrIdent => {
                self.start_temp_vec(Asn1Tag::XMLInteger);
                self.next(&[TokenKind::Number, TokenKind::ValueRefOrIdent])?;
                self.end_temp_vec(Asn1Tag::XMLInteger);
            }
            TokenKind::ForwardSlash => {
                self.xml_iri()?;
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
            TokenKind::TypeOrModuleRef,
            TokenKind::XMLAsn1TypeName,
        ])?;

        if tok.kind == TokenKind::Underscore {
            self.next(&[TokenKind::Underscore])?;
        }

        let tok = self.next(&[TokenKind::TypeOrModuleRef, TokenKind::XMLAsn1TypeName])?;
        if tok.kind == TokenKind::TypeOrModuleRef {
            let tok = self.peek(&[
                TokenKind::Dot,
                TokenKind::Greater,
                TokenKind::XMLSingleTagEnd,
            ])?;
            if tok.kind == TokenKind::Dot {
                self.next(&[TokenKind::Dot])?;
                self.next(&[TokenKind::TypeOrModuleRef])?;
            }
        }

        self.end_temp_vec(Asn1Tag::XMLTypedValue);
        Ok(())
    }

    /// Parse an internationalised resource identifier
    fn xml_iri(&mut self) -> Result<()> {
        self.start_temp_vec(Asn1Tag::XMLIri);

        self.next(&[TokenKind::ForwardSlash])?;
        self.next(&[
            TokenKind::IntegerUnicodeLabel,
            TokenKind::NonIntegerUnicodeLabel,
        ])?;

        loop {
            let next = self.peek(&[TokenKind::XMLEndTag, TokenKind::ForwardSlash])?;
            if next.kind == TokenKind::XMLEndTag {
                break;
            }
            self.next(&[TokenKind::ForwardSlash])?;
            self.next(&[
                TokenKind::IntegerUnicodeLabel,
                TokenKind::NonIntegerUnicodeLabel,
            ])?;
        }

        self.end_temp_vec(Asn1Tag::XMLIri);

        Ok(())
    }
}*/

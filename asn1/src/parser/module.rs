use crate::{cst::Asn1Tag, token::TokenKind};

use super::{ty::TypeStartKind, Parser, Result};

impl<'a> Parser<'a> {
    /// Parse a single ASN.1 module definition
    pub(in crate::parser) fn module_definition(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::ModuleDefinition);

        self.module_identifier()?;
        self.next(&[TokenKind::KwDefinitions])?;
        self.module_defaults()?;
        self.next(&[TokenKind::Assignment])?;
        self.next(&[TokenKind::KwBegin])?;

        self.exports()?;
        // TODO: imports

        // ensure there is at least one assignment before the end token
        self.peek(&[TokenKind::TypeReference, TokenKind::ValueReference])?;
        loop {
            self.assignment()?;

            if self.next(&[TokenKind::KwEnd]).is_ok() {
                break;
            }
        }

        // TODO: encoding control sections

        self.end_temp_vec(Asn1Tag::ModuleDefinition);
        Ok(())
    }

    /// Identifier at the start of a module
    fn module_identifier(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::ModuleIdentifier);

        self.next(&[TokenKind::ModuleReference])?;

        let tok = self.peek(&[TokenKind::LeftCurly, TokenKind::KwDefinitions])?;
        if tok.kind == TokenKind::LeftCurly {
            self.definitive_oid()?;

            let tok = self.peek(&[TokenKind::DoubleQuote, TokenKind::KwDefinitions])?;
            if tok.kind == TokenKind::DoubleQuote {
                self.iri_value()?;
            }
        }

        self.end_temp_vec(Asn1Tag::ModuleIdentifier);
        Ok(())
    }

    fn definitive_oid(&mut self) -> Result {
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
        Ok(())
    }

    /// The bit between the `DEFINITIONS` keyword and the assignment
    fn module_defaults(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::ModuleDefaults);

        self.encoding_reference_default()?;
        self.tag_default()?;
        self.extension_default()?;

        self.end_temp_vec(Asn1Tag::ModuleDefaults);
        Ok(())
    }

    fn encoding_reference_default(&mut self) -> Result {
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
        Ok(())
    }

    fn tag_default(&mut self) -> Result {
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
        Ok(())
    }

    fn extension_default(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::ExtensionDefault);
        self.peek(&[TokenKind::KwExtensibility, TokenKind::Assignment])?;
        if self.next(&[TokenKind::KwExtensibility]).is_ok() {
            self.next(&[TokenKind::KwImplied])?;
        }
        self.end_temp_vec(Asn1Tag::ExtensionDefault);

        Ok(())
    }

    /// Exported symbols section
    fn exports(&mut self) -> Result {
        let tok = self.peek(&[
            TokenKind::KwExports,
            TokenKind::TypeReference,
            TokenKind::ValueReference,
        ])?;

        if tok.kind != TokenKind::KwExports {
            return Ok(());
        }
        self.start_temp_vec(Asn1Tag::Exports);

        self.next(&[TokenKind::KwExports])?;
        let tok = self.peek(&[
            TokenKind::SemiColon,
            TokenKind::KwAll,
            TokenKind::ValueReference,
            TokenKind::TypeReference,
        ])?;
        if tok.kind == TokenKind::KwAll {
            self.next(&[TokenKind::KwAll])?;
        } else if tok.kind != TokenKind::SemiColon {
            self.symbol_list()?;
        }

        self.next(&[TokenKind::SemiColon])?;

        self.end_temp_vec(Asn1Tag::Exports);

        Ok(())
    }

    /// Parse a single assignment to a name
    fn assignment(&mut self) -> Result {
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
}

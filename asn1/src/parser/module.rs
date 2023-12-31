use crate::{cst::Asn1Tag, token::TokenKind};

use super::{reference::SymbolListKind, type_or_value::TypeOrValue, Parser, Result};

impl<'a> Parser<'a> {
    /// Parse a single ASN.1 module definition
    pub(super) fn module_definition(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::ModuleDefinition)?;

        self.module_identifier()?;
        self.next(&[TokenKind::KwDefinitions])?;
        self.module_defaults()?;
        self.next(&[TokenKind::Assignment])?;
        self.next(&[TokenKind::KwBegin])?;

        self.exports()?;
        self.imports()?;

        // ensure there is at least one assignment before the end token
        self.peek(&[TokenKind::TypeOrModuleRef, TokenKind::ValueRefOrIdent])?;
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
        self.start_temp_vec(Asn1Tag::ModuleIdentifier)?;

        self.next(&[TokenKind::TypeOrModuleRef])?;

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
        self.start_temp_vec(Asn1Tag::DefinitiveOID)?;
        self.next(&[TokenKind::LeftCurly])?;

        let mut kind = &[TokenKind::ValueRefOrIdent, TokenKind::Number][..];

        loop {
            let tok = self.next(kind)?;
            if tok.kind == TokenKind::RightCurly {
                break;
            }

            if tok.kind == TokenKind::ValueRefOrIdent && self.next(&[TokenKind::LeftParen]).is_ok()
            {
                self.next(&[TokenKind::Number])?;
                self.next(&[TokenKind::RightParen])?;
            }

            kind = &[
                TokenKind::ValueRefOrIdent,
                TokenKind::Number,
                TokenKind::RightCurly,
            ];
        }

        self.end_temp_vec(Asn1Tag::DefinitiveOID);
        Ok(())
    }

    /// The bit between the `DEFINITIONS` keyword and the assignment
    fn module_defaults(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::ModuleDefaults)?;

        self.encoding_reference_default()?;
        self.tag_default()?;
        self.extension_default()?;

        self.end_temp_vec(Asn1Tag::ModuleDefaults);
        Ok(())
    }

    fn encoding_reference_default(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::EncodingReferenceDefault)?;
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
        self.start_temp_vec(Asn1Tag::TagDefault)?;
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
        self.start_temp_vec(Asn1Tag::ExtensionDefault)?;
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
            TokenKind::KwImports,
            TokenKind::TypeOrModuleRef,
            TokenKind::ValueRefOrIdent,
        ])?;

        if tok.kind != TokenKind::KwExports {
            return Ok(());
        }
        self.start_temp_vec(Asn1Tag::Exports)?;

        self.next(&[TokenKind::KwExports])?;
        let tok = self.peek(&[
            TokenKind::SemiColon,
            TokenKind::KwAll,
            TokenKind::ValueRefOrIdent,
            TokenKind::TypeOrModuleRef,
        ])?;
        if tok.kind == TokenKind::KwAll {
            self.next(&[TokenKind::KwAll])?;
        } else if tok.kind != TokenKind::SemiColon {
            self.symbol_list(SymbolListKind::Exports)?;
        }

        self.next(&[TokenKind::SemiColon])?;

        self.end_temp_vec(Asn1Tag::Exports);

        Ok(())
    }

    /// Parse the list of imported symbols
    fn imports(&mut self) -> Result {
        let tok = self.peek(&[
            TokenKind::KwImports,
            TokenKind::TypeOrModuleRef,
            TokenKind::ValueRefOrIdent,
        ])?;

        if tok.kind != TokenKind::KwImports {
            return Ok(());
        }
        self.start_temp_vec(Asn1Tag::Imports)?;

        self.next(&[TokenKind::KwImports])?;
        self.symbols_imported()?;
        self.next(&[TokenKind::SemiColon])?;

        self.end_temp_vec(Asn1Tag::Imports);

        Ok(())
    }

    /// Parse a single assignment to a name
    fn assignment(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::Assignment)?;

        let name = self.next(&[
            TokenKind::TypeOrModuleRef,
            TokenKind::ValueRefOrIdent,
            TokenKind::KwEnd,
        ])?;

        // TODO: Object class assignment
        // TODO: Object set assignment
        // TODO: Parameterized assignment

        match name.kind {
            TokenKind::KwEnd => {
                // shouldn't get here but oh well, end is in the list so that
                // better expected lines can be generated
                return Ok(());
            }
            TokenKind::TypeOrModuleRef => {
                self.start_temp_vec(Asn1Tag::TypeAssignment)?;

                let ty = self.type_or_value(TypeOrValue {
                    is_type: true,
                    is_value: false,
                    alternative: &[TokenKind::Assignment],
                    subsequent: &[TokenKind::Assignment],
                })?;

                self.next(&[TokenKind::Assignment])?;

                if ty.is_assign() {
                    self.type_or_value(TypeOrValue {
                        is_type: true,
                        is_value: false,
                        alternative: &[],
                        subsequent: &[
                            TokenKind::TypeOrModuleRef,
                            TokenKind::ValueRefOrIdent,
                            TokenKind::KwEnd,
                        ],
                    })?;
                } else {
                    self.next(&[TokenKind::LeftCurly])?;
                    // TODO: element set specs
                    self.next(&[TokenKind::RightCurly])?;
                }

                self.end_temp_vec(Asn1Tag::TypeAssignment);
            }
            TokenKind::ValueRefOrIdent => {
                self.start_temp_vec(Asn1Tag::ValueAssignment)?;

                let ty = self.type_or_value(TypeOrValue {
                    is_type: true,
                    is_value: false,
                    alternative: &[TokenKind::Assignment],
                    subsequent: &[TokenKind::Assignment],
                })?;
                self.next(&[TokenKind::Assignment])?;

                if ty.is_assign() {
                    self.xml_typed_value()?;
                } else {
                    self.type_or_value(TypeOrValue {
                        is_type: false,
                        is_value: true,
                        alternative: &[],
                        subsequent: &[
                            TokenKind::TypeOrModuleRef,
                            TokenKind::ValueRefOrIdent,
                            TokenKind::KwEnd,
                        ],
                    })?;
                }
                self.end_temp_vec(Asn1Tag::ValueAssignment)
            }
            _ => panic!("try consume error"),
        }

        self.end_temp_vec(Asn1Tag::Assignment);

        Ok(())
    }
}

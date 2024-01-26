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

            let tok = self.peek(&[
                TokenKind::TypeOrModuleRef,
                TokenKind::ValueRefOrIdent,
                TokenKind::KwEnd,
                TokenKind::KwEncodingControl,
            ])?;
            if tok.kind == TokenKind::KwEnd || tok.kind == TokenKind::KwEncodingControl {
                break;
            }
        }

        let tok = self.peek(&[TokenKind::KwEnd, TokenKind::KwEncodingControl])?;
        if tok.kind == TokenKind::KwEncodingControl {
            self.encoding_control()?;
        }

        self.next(&[TokenKind::KwEnd])?;

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

            let tok = self.peek(&[TokenKind::CString, TokenKind::KwDefinitions])?;
            if tok.kind == TokenKind::CString {
                self.next(&[TokenKind::CString])?;
            }
        }

        self.end_temp_vec(Asn1Tag::ModuleIdentifier);
        Ok(())
    }

    /// Object identifier after the name of a module
    /// - no local variables in scope
    /// - we know it must be an OID so don't just parse it as a braced value
    fn definitive_oid(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::DefinitiveOID)?;
        self.next(&[TokenKind::LeftCurly])?;

        loop {
            self.definitive_oid_component()?;
            let tok = self.peek(&[
                TokenKind::ValueRefOrIdent,
                TokenKind::Number,
                TokenKind::RightCurly,
            ])?;
            if tok.kind == TokenKind::RightCurly {
                self.next(&[TokenKind::RightCurly])?;
                break;
            }
        }

        self.end_temp_vec(Asn1Tag::DefinitiveOID);
        Ok(())
    }

    /// Single component of the object identifier after the name of a module
    fn definitive_oid_component(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::DefinitiveOIDComponent)?;

        let tok = self.next(&[TokenKind::ValueRefOrIdent, TokenKind::Number])?;

        if tok.kind == TokenKind::ValueRefOrIdent {
            let tok = self.peek(&[
                TokenKind::LeftParen,
                TokenKind::ValueRefOrIdent,
                TokenKind::Number,
                TokenKind::RightCurly,
            ])?;
            if tok.kind == TokenKind::LeftParen {
                self.next(&[TokenKind::LeftParen])?;
                self.next(&[TokenKind::Number])?;
                self.next(&[TokenKind::RightParen])?;
            }
        }

        self.end_temp_vec(Asn1Tag::DefinitiveOIDComponent);
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

    /// Parse an encoding reference specifier.
    fn encoding_reference_default(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::EncodingReferenceDefault)?;
        let tok = self.peek(&[
            TokenKind::TypeOrModuleRef,
            TokenKind::KwExplicit,
            TokenKind::KwImplicit,
            TokenKind::KwAutomatic,
            TokenKind::KwExtensibility,
            TokenKind::Assignment,
        ])?;
        if tok.kind == TokenKind::TypeOrModuleRef {
            self.next(&[TokenKind::TypeOrModuleRef])?;
            self.next(&[TokenKind::KwInstructions])?;
        }
        self.end_temp_vec(Asn1Tag::EncodingReferenceDefault);
        Ok(())
    }

    /// Parse `EXPLICIT TAGS` or `IMPLICIT TAGS` or `AUTOMATIC TAGS` or none
    fn tag_default(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::TagDefault)?;
        let tok = self.peek(&[
            TokenKind::KwExplicit,
            TokenKind::KwImplicit,
            TokenKind::KwAutomatic,
            TokenKind::KwExtensibility,
            TokenKind::Assignment,
        ])?;
        if matches!(
            tok.kind,
            TokenKind::KwExplicit | TokenKind::KwImplicit | TokenKind::KwAutomatic
        ) {
            self.next(&[
                TokenKind::KwExplicit,
                TokenKind::KwImplicit,
                TokenKind::KwAutomatic,
            ])?;
            self.next(&[TokenKind::KwTags])?;
        }
        self.end_temp_vec(Asn1Tag::TagDefault);
        Ok(())
    }

    /// Parse `EXTENSIBILITY IMPLIED` or none
    fn extension_default(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::ExtensionDefault)?;
        let tok = self.peek(&[TokenKind::KwExtensibility, TokenKind::Assignment])?;
        if tok.kind == TokenKind::KwExtensibility {
            self.next(&[TokenKind::KwExtensibility])?;
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

    /// Parse the list of imported symbols by skipping over everything
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
            TokenKind::KwEncodingControl,
        ])?;

        // TODO: Object class assignment
        // TODO: Object set assignment
        // TODO: Parameterized assignment

        match name.kind {
            TokenKind::TypeOrModuleRef => {
                self.start_temp_vec(Asn1Tag::TypeAssignment)?;

                let ty = self.type_or_value(TypeOrValue {
                    alternative: &[TokenKind::Assignment],
                    subsequent: &[TokenKind::Assignment],
                })?;

                self.next(&[TokenKind::Assignment])?;

                if ty.is_assign() {
                    self.type_or_value(TypeOrValue {
                        alternative: &[],
                        subsequent: &[
                            TokenKind::TypeOrModuleRef,
                            TokenKind::ValueRefOrIdent,
                            TokenKind::KwEnd,
                            TokenKind::KwEncodingControl,
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
                    alternative: &[TokenKind::Assignment],
                    subsequent: &[TokenKind::Assignment],
                })?;
                self.next(&[TokenKind::Assignment])?;

                if ty.is_assign() {
                    self.xml_value()?;
                } else {
                    self.type_or_value(TypeOrValue {
                        alternative: &[],
                        subsequent: &[
                            TokenKind::TypeOrModuleRef,
                            TokenKind::ValueRefOrIdent,
                            TokenKind::KwEnd,
                            TokenKind::KwEncodingControl,
                        ],
                    })?;
                }
                self.end_temp_vec(Asn1Tag::ValueAssignment)
            }
            _ => (),
        }

        self.end_temp_vec(Asn1Tag::Assignment);

        Ok(())
    }

    /// Parse a list of encoding control sections
    fn encoding_control(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::EncodingControl)?;

        loop {
            self.encoding_control_section()?;

            let tok = self.peek(&[TokenKind::KwEncodingControl, TokenKind::KwEnd])?;
            if tok.kind == TokenKind::KwEnd {
                break;
            }
        }

        self.end_temp_vec(Asn1Tag::EncodingControl);
        Ok(())
    }

    /// Parse the inside of a single encoding control section
    fn encoding_control_section(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::EncodingControlSection)?;

        self.next(&[TokenKind::KwEncodingControl])?;

        loop {
            let tok = self.peek(&[])?;
            if tok.kind == TokenKind::KwEnd || tok.kind == TokenKind::KwEncodingControl {
                break;
            }
            self.next(&[])?;
        }

        self.end_temp_vec(Asn1Tag::EncodingControlSection);
        Ok(())
    }
}

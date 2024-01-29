//! sequence, set, and choice type parsing

use crate::{cst::Asn1Tag, token::TokenKind};

use super::{Parser, Result, TypeOrValue};

impl<'a> Parser<'a> {
    /// Parse a sequence type definition
    pub(super) fn sequence_type(&mut self, expecting: TypeOrValue) -> Result {
        self.sequence_set(expecting, Asn1Tag::SequenceType, TokenKind::KwSequence)
    }

    /// Parse a set type definition
    pub(super) fn set_type(&mut self, expecting: TypeOrValue) -> Result {
        self.sequence_set(expecting, Asn1Tag::SetType, TokenKind::KwSet)
    }

    // Parse either a sequence or set type
    fn sequence_set(&mut self, expecting: TypeOrValue, tag: Asn1Tag, kind: TokenKind) -> Result {
        self.start_temp_vec(tag)?;

        self.next(vec![kind])?;

        let tok = self.peek(&[TokenKind::LeftCurly, TokenKind::KwOf])?;
        if tok.kind == TokenKind::LeftCurly {
            self.struct_type()?;
        } else {
            self.next(&[TokenKind::KwOf])?;
            self.type_or_value_named(TypeOrValue {
                alternative: &[],
                subsequent: expecting.subsequent,
            })?;
        }

        self.end_temp_vec(tag);

        self.open_type_field_value(expecting)?;

        Ok(())
    }

    /// Parse the inside of a set or sequence type, including the curly braces.
    /// Does not deal with set of/sequence of/constrained types.
    /// ```bnf
    /// ComponentTypeLists ::=
    ///     RootComponentTypeList
    ///   | RootComponentTypeList "," ExtensionAndException ExtensionAdditions
    ///         OptionalExtensionMarker
    ///   | RootComponentTypeList "," ExtensionAndException ExtensionAdditions
    ///         ExtensionEndMarker "," RootComponentTypeList
    ///   | ExtensionAndException ExtensionAdditions ExtensionEndMarker ","
    ///         RootComponentTypeList
    ///   | ExtensionAndException ExtensionAdditions OptionalExtensionMarker
    /// ```
    fn struct_type(&mut self) -> Result {
        self.next(&[TokenKind::LeftCurly])?;

        let tok = self.peek(&[
            TokenKind::Ellipsis,
            TokenKind::RightCurly,
            TokenKind::ValueRefOrIdent,
            TokenKind::KwComponents,
        ])?;
        match tok.kind {
            TokenKind::Ellipsis => self.extension_struct()?,
            TokenKind::ValueRefOrIdent | TokenKind::KwComponents => self.component_struct()?,
            _ => (),
        }

        self.next(&[TokenKind::RightCurly])?;

        Ok(())
    }

    /// Parse the contents of a set or sequence type where the value starts with
    /// a component type.
    fn component_struct(&mut self) -> Result {
        let is_comma = self.component_type_list(&[TokenKind::RightCurly])?;
        if !is_comma {
            return Ok(());
        }

        self.extension_struct()
    }

    /// Parse the contents of a set or sequence type where the value starts with
    /// an extension marker.
    fn extension_struct(&mut self) -> Result {
        self.extension_and_exception()?;

        let tok = self.peek(&[TokenKind::RightCurly, TokenKind::Comma])?;
        if tok.kind == TokenKind::RightCurly {
            return Ok(());
        }

        self.next(&[TokenKind::Comma])?;

        let tok = self.peek(&[
            TokenKind::ValueRefOrIdent,
            TokenKind::KwComponents,
            TokenKind::VersionOpen,
            TokenKind::Ellipsis,
        ])?;
        if tok.kind != TokenKind::Ellipsis {
            let is_comma = self.extension_additions()?;
            if !is_comma {
                return Ok(());
            }
        }

        self.next(&[TokenKind::Ellipsis])?;

        let tok = self.peek(&[TokenKind::RightCurly, TokenKind::Comma])?;
        if tok.kind == TokenKind::RightCurly {
            return Ok(());
        }

        self.next(&[TokenKind::Comma])?;
        self.component_type_list(&[TokenKind::RightCurly])?;

        Ok(())
    }

    /// Parse the list of versioned additions to a struct type.
    /// `[[0: my INTEGER]], a INTEGER, [[1: new BOOLEAN]]`  Returns true if a
    /// trailing comma was consumed.
    fn extension_additions(&mut self) -> Result<bool> {
        self.start_temp_vec(Asn1Tag::ExtensionAdditions)?;

        let mut ret = false;
        loop {
            self.extension_addition()?;
            let tok = self.peek(&[TokenKind::Comma, TokenKind::RightCurly])?;
            if tok.kind != TokenKind::Comma {
                break;
            }
            self.next(&[TokenKind::Comma])?;

            let tok = self.peek(&[
                TokenKind::ValueRefOrIdent,
                TokenKind::KwComponents,
                TokenKind::VersionOpen,
                TokenKind::Ellipsis,
            ])?;
            if tok.kind == TokenKind::Ellipsis {
                ret = true;
                break;
            }
        }

        self.end_temp_vec(Asn1Tag::ExtensionAdditions);

        Ok(ret)
    }

    /// Parse a single extension addition
    fn extension_addition(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::ExtensionAddition)?;

        let tok = self.peek(&[
            TokenKind::ValueRefOrIdent,
            TokenKind::KwComponents,
            TokenKind::VersionOpen,
        ])?;
        if tok.kind == TokenKind::VersionOpen {
            self.extension_addition_group()?;
        } else {
            self.component_type(&[TokenKind::Comma, TokenKind::RightCurly])?;
        }

        self.end_temp_vec(Asn1Tag::ExtensionAddition);

        Ok(())
    }

    /// Parse a single versioned addition to a struct type. `[[0: my INTEGER]]`
    fn extension_addition_group(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::ExtensionAdditionGroup)?;

        self.next(&[TokenKind::VersionOpen])?;

        let tok = self.peek(&[
            TokenKind::ValueRefOrIdent,
            TokenKind::KwComponents,
            TokenKind::Number,
        ])?;
        if tok.kind == TokenKind::Number {
            self.version_number()?;
        }

        self.component_type_list(&[TokenKind::VersionClose])?;
        self.next(&[TokenKind::VersionClose])?;

        self.end_temp_vec(Asn1Tag::ExtensionAdditionGroup);
        Ok(())
    }

    /// Parse a component type version number e.g. `1:`
    fn version_number(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::VersionNumber)?;

        self.next(&[TokenKind::Number])?;
        self.next(&[TokenKind::Colon])?;

        self.end_temp_vec(Asn1Tag::VersionNumber);

        Ok(())
    }

    /// Parse the extension marker an an optional exception specification
    fn extension_and_exception(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::ExtensionAndException)?;

        self.next(&[TokenKind::Ellipsis])?;
        self.exception_spec(&[TokenKind::Comma, TokenKind::RightCurly])?;

        self.end_temp_vec(Asn1Tag::ExtensionAndException);

        Ok(())
    }

    /// Parse components of a struct type.  Returns true if a comma was consumed
    /// that does not have another component type after it.
    fn component_type_list(&mut self, subsequent: &[TokenKind]) -> Result<bool> {
        self.start_temp_vec(Asn1Tag::ComponentTypeList)?;

        let mut subsequent = subsequent.to_vec();
        subsequent.push(TokenKind::Comma);

        let mut ret = false;
        loop {
            self.component_type(&subsequent)?;
            let tok = self.peek(&[])?;
            if tok.kind != TokenKind::Comma {
                break;
            }
            self.next(&[TokenKind::Comma])?;

            let tok = self.peek(&[])?;
            if tok.kind != TokenKind::KwComponents && tok.kind != TokenKind::ValueRefOrIdent {
                ret = true;
                break;
            }
        }

        self.end_temp_vec(Asn1Tag::ComponentTypeList);

        Ok(ret)
    }

    /// Parse a single component of a struct type
    fn component_type(&mut self, subsequent: &[TokenKind]) -> Result {
        self.start_temp_vec(Asn1Tag::ComponentType)?;

        let tok = self.next(&[TokenKind::ValueRefOrIdent, TokenKind::KwComponents])?;
        if tok.kind == TokenKind::KwComponents {
            self.next(&[TokenKind::KwOf])?;
            self.type_or_value(TypeOrValue {
                alternative: &[],
                subsequent,
            })?;
        } else {
            let mut ty_subsequent = subsequent.to_vec();
            ty_subsequent.push(TokenKind::KwOptional);
            ty_subsequent.push(TokenKind::KwDefault);

            self.type_or_value(TypeOrValue {
                alternative: &[],
                subsequent: &ty_subsequent,
            })?;

            let tok = self.peek(ty_subsequent)?;
            if tok.kind == TokenKind::KwOptional {
                self.next(&[])?;
            } else if tok.kind == TokenKind::KwDefault {
                self.next(&[])?;
                self.type_or_value(TypeOrValue {
                    alternative: &[],
                    subsequent,
                })?;
            }
        }

        self.end_temp_vec(Asn1Tag::ComponentType);
        Ok(())
    }
}

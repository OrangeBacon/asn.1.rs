use crate::{cst::Asn1Tag, token::TokenKind};

use super::{Parser, Result};

impl<'a> Parser<'a> {
    /// parse reference to defined value
    pub(in crate::parser) fn defined_value(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::DefinedValue)?;

        // TODO: parameterized value

        let tok = self.peek(&[TokenKind::ValueRefOrIdent, TokenKind::TypeOrModuleRef])?;
        if tok.kind == TokenKind::TypeOrModuleRef {
            self.external_value_reference()?;
        } else {
            self.next(&[TokenKind::ValueRefOrIdent])?;
        }

        self.end_temp_vec(Asn1Tag::DefinedValue);
        Ok(())
    }

    pub(super) fn integer_value(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::IntegerValue)?;

        let tok = self.next(&[TokenKind::Number, TokenKind::Hyphen])?;

        if tok.kind == TokenKind::Hyphen {
            self.next(&[TokenKind::Number])?;
        }

        self.end_temp_vec(Asn1Tag::IntegerValue);
        Ok(())
    }

    /// Parse a reference to an external value
    fn external_value_reference(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::ExternalValueReference)?;

        self.next(&[TokenKind::TypeOrModuleRef])?;
        self.next(&[TokenKind::Dot])?;
        self.next(&[TokenKind::ValueRefOrIdent])?;

        self.end_temp_vec(Asn1Tag::ExternalValueReference);
        Ok(())
    }

    /// Parse an object identifier value
    pub(in crate::parser) fn object_identifier_value(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::ObjectIDValue)?;

        self.next(&[TokenKind::LeftCurly])?;

        loop {
            self.object_id_component()?;
            let tok = self.peek(&[
                TokenKind::Number,
                TokenKind::ValueRefOrIdent,
                TokenKind::TypeOrModuleRef,
                TokenKind::RightCurly,
            ])?;
            if tok.kind == TokenKind::RightCurly {
                break;
            }
        }

        self.next(&[TokenKind::RightCurly])?;

        self.end_temp_vec(Asn1Tag::ObjectIDValue);
        Ok(())
    }

    /// parse a single object ID component, assuming the next token is not a
    /// closing curly brace.
    ///
    /// Object ID component =
    /// | ident
    /// | number
    /// | ident(number)
    /// | ident(defined value)
    /// | defined value
    ///
    /// Ident and value reference are the same token, therefore if one of them
    /// matches it could be ambiguous, so we assume it always takes the ident
    /// option and the value reference part of defined value never matches in
    /// this function.
    fn object_id_component(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::ObjectIDComponent)?;

        let tok = self.peek(&[
            TokenKind::ValueRefOrIdent,
            TokenKind::Number,
            TokenKind::ValueRefOrIdent,
            TokenKind::TypeOrModuleRef,
            TokenKind::RightCurly,
        ])?;

        match tok.kind {
            TokenKind::Number => {
                self.next(&[TokenKind::Number])?;
            }
            TokenKind::TypeOrModuleRef => {
                self.defined_value()?;
            }
            _ => {
                self.next(&[TokenKind::ValueRefOrIdent])?;
                let tok = self.peek(&[
                    TokenKind::LeftParen,
                    TokenKind::ValueRefOrIdent,
                    TokenKind::Number,
                    TokenKind::ValueRefOrIdent,
                    TokenKind::TypeOrModuleRef,
                    TokenKind::RightCurly,
                ])?;
                if tok.kind == TokenKind::LeftParen {
                    self.next(&[TokenKind::LeftParen])?;
                    let tok = self.peek(&[
                        TokenKind::TypeOrModuleRef,
                        TokenKind::ValueRefOrIdent,
                        TokenKind::Number,
                    ])?;
                    if tok.kind == TokenKind::Number {
                        self.next(&[TokenKind::Number])?;
                    } else {
                        self.defined_value()?;
                    }
                    self.next(&[TokenKind::RightParen])?;
                }
            }
        }

        self.end_temp_vec(Asn1Tag::ObjectIDComponent);
        Ok(())
    }
}

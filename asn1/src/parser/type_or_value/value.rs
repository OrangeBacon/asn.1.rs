use crate::{cst::Asn1Tag, token::TokenKind};

use super::{Parser, Result, TypeOrValue};

impl<'a> Parser<'a> {
    /// Parse a numeric value, either integer or real (floating point)
    pub(super) fn number_value(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::NumberValue)?;

        let tok = self.next(&[TokenKind::Number, TokenKind::Hyphen])?;

        if tok.kind == TokenKind::Hyphen {
            self.next(&[TokenKind::Number])?;
        }

        self.end_temp_vec(Asn1Tag::NumberValue);
        Ok(())
    }

    /// Parse a value starting and ending with curly braces.  Many different types
    /// have values between curly braces, where the correct parse cannot be
    /// determined until the type is known.  This is especially the case where
    /// class objects with custom syntax are defined.  Therefore, this parser will
    /// parse all matching values as the same flat list of tokens, to then be
    /// re-parsed later depending on the type of the value.
    pub(super) fn braced_value(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::BracedValue)?;

        self.next(&[TokenKind::LeftCurly])?;

        let mut depth = 1;
        loop {
            let tok = self.next(&[])?;
            if tok.kind == TokenKind::RightCurly {
                depth -= 1;
            } else if tok.kind == TokenKind::LeftCurly {
                depth += 1;
            }
            if depth == 0 {
                break;
            }
        }

        self.end_temp_vec(Asn1Tag::BracedValue);
        Ok(())
    }

    /// Parse the value of a choice type, assuming that the initial identifier
    /// has already been matched.
    /// ```asn1
    /// ChoiceValue ::= identifier ':' Value
    /// ```
    pub(super) fn choice_value(&mut self, subsequent: &[TokenKind]) -> Result {
        self.start_temp_vec(Asn1Tag::ChoiceValue)?;

        self.next(&[TokenKind::Colon])?;

        self.type_or_value(TypeOrValue {
            subsequent,
            alternative: &[],
        })?;

        self.end_temp_vec(Asn1Tag::ChoiceValue);
        Ok(())
    }

    /// Parse the `CONTAINING Value` options within bit string and octet string
    pub(super) fn containing_value(&mut self, subsequent: &[TokenKind]) -> Result {
        self.start_temp_vec(Asn1Tag::ContainingValue)?;

        self.next(&[TokenKind::KwContaining])?;

        self.type_or_value(TypeOrValue {
            subsequent,
            alternative: &[],
        })?;

        self.end_temp_vec(Asn1Tag::ContainingValue);
        Ok(())
    }

    /// Parse `Type : Value` syntax, starting with the colon
    pub(super) fn open_type_field_value(&mut self, expecting: TypeOrValue) -> Result {
        let mut kind = expecting.subsequent.to_vec();
        kind.push(TokenKind::Colon);
        let tok = self.peek(kind)?;
        if tok.kind != TokenKind::Colon {
            return Ok(());
        }

        self.start_temp_vec(Asn1Tag::OpenTypeFieldValue)?;

        self.next(&[TokenKind::Colon])?;
        self.type_or_value(TypeOrValue {
            subsequent: expecting.subsequent,
            alternative: &[],
        })?;

        self.end_temp_vec(Asn1Tag::OpenTypeFieldValue);
        Ok(())
    }
}

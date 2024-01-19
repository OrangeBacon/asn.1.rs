use crate::{cst::Asn1Tag, token::TokenKind};

use super::{Parser, Result};

impl<'a> Parser<'a> {
    pub(super) fn integer_value(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::IntegerValue)?;

        let tok = self.next(&[TokenKind::Number, TokenKind::Hyphen])?;

        if tok.kind == TokenKind::Hyphen {
            self.next(&[TokenKind::Number])?;
        }

        self.end_temp_vec(Asn1Tag::IntegerValue);
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
}

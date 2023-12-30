use crate::{cst::Asn1Tag, token::TokenKind};

use super::{Parser, Result};

impl<'a> Parser<'a> {
    pub(in crate::parser) fn symbol_list(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::SymbolList);

        loop {
            self.symbol()?;
            let tok = self.peek(&[TokenKind::Comma, TokenKind::SemiColon])?;
            if tok.kind != TokenKind::Comma {
                break;
            }
            self.next(&[TokenKind::Comma])?;
        }

        self.end_temp_vec(Asn1Tag::SymbolList);

        Ok(())
    }

    /// Reference or parameterized reference.
    fn symbol(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::Symbol);

        self.reference()?;

        let tok = self.peek(&[TokenKind::LeftCurly, TokenKind::Comma, TokenKind::SemiColon])?;
        if tok.kind == TokenKind::LeftCurly {
            self.next(&[TokenKind::LeftCurly])?;
            self.next(&[TokenKind::RightCurly])?;
        }

        self.end_temp_vec(Asn1Tag::Symbol);
        Ok(())
    }

    /// Either type or value reference
    fn reference(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::Reference);

        // object references all parse as a type reference, so this is all that
        // needs to be specified here
        self.next(&[TokenKind::TypeReference, TokenKind::ValueReference])?;

        self.end_temp_vec(Asn1Tag::Reference);
        Ok(())
    }
}

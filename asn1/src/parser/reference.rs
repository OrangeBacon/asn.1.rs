use crate::{cst::Asn1Tag, token::TokenKind};

use super::{Parser, Result};

/// Location that a symbol list is being parsed in, so that the next token can
/// be peeked successfully.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum SymbolListKind {
    Exports,
}

impl<'a> Parser<'a> {
    /// List of symbols within an import or export statement
    pub(super) fn symbol_list(&mut self, next: SymbolListKind) -> Result {
        self.start_temp_vec(Asn1Tag::SymbolList)?;

        let kind = match next {
            SymbolListKind::Exports => &[TokenKind::Comma, TokenKind::SemiColon],
        };

        loop {
            self.symbol(next)?;
            let tok = self.peek(kind)?;
            if tok.kind != TokenKind::Comma {
                break;
            }
            self.next(&[TokenKind::Comma])?;
        }

        self.end_temp_vec(Asn1Tag::SymbolList);

        Ok(())
    }

    /// Reference or parameterized reference.
    fn symbol(&mut self, next: SymbolListKind) -> Result {
        self.start_temp_vec(Asn1Tag::Symbol)?;

        self.reference()?;

        let kind = match next {
            SymbolListKind::Exports => {
                &[TokenKind::LeftCurly, TokenKind::Comma, TokenKind::SemiColon]
            }
        };

        let tok = self.peek(kind)?;
        if tok.kind == TokenKind::LeftCurly {
            self.next(&[TokenKind::LeftCurly])?;
            self.next(&[TokenKind::RightCurly])?;
        }

        self.end_temp_vec(Asn1Tag::Symbol);
        Ok(())
    }

    /// Either type or value reference
    fn reference(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::Reference)?;

        // object references all parse as a type reference, so this is all that
        // needs to be specified here
        self.next(&[TokenKind::TypeOrModuleRef, TokenKind::ValueRefOrIdent])?;

        self.end_temp_vec(Asn1Tag::Reference);
        Ok(())
    }

    /// Parse the references within an import statement.  Does not try to create
    /// a tree, just accepts any token that could appear between the import
    /// keyword and the final semi-colon, to be parsed properly later.
    pub(super) fn symbols_imported(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::SymbolsFromModuleList)?;

        let kinds = &[
            TokenKind::TypeOrModuleRef,
            TokenKind::ValueRefOrIdent,
            TokenKind::SemiColon,
            TokenKind::Comma,
            TokenKind::KwFrom,
            TokenKind::KwWith,
            TokenKind::Dot,
            TokenKind::Number,
            TokenKind::LeftCurly,
            TokenKind::RightCurly,
            TokenKind::LeftParen,
            TokenKind::RightParen,
        ];

        loop {
            let tok = self.peek(kinds)?;
            if tok.kind == TokenKind::SemiColon {
                break;
            }
            self.next(kinds)?;
        }

        self.end_temp_vec(Asn1Tag::SymbolsFromModuleList);
        Ok(())
    }
}

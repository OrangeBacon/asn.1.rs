/*use crate::{cst::Asn1Tag, token::TokenKind};

use super::{Parser, Result};

/// Location that a symbol list is being parsed in, so that the next token can
/// be peeked successfully.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum SymbolListKind {
    Imports,
    Exports,
}

impl<'a> Parser<'a> {
    /// List of symbols within an import or export statement
    pub(super) fn symbol_list(&mut self, next: SymbolListKind) -> Result {
        self.start_temp_vec(Asn1Tag::SymbolList);

        let kind = match next {
            SymbolListKind::Imports => &[TokenKind::Comma, TokenKind::KwFrom],
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
        self.start_temp_vec(Asn1Tag::Symbol);

        self.reference()?;

        let kind = match next {
            SymbolListKind::Imports => &[TokenKind::LeftCurly, TokenKind::Comma, TokenKind::KwFrom],
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
        self.start_temp_vec(Asn1Tag::Reference);

        // object references all parse as a type reference, so this is all that
        // needs to be specified here
        self.next(&[TokenKind::TypeOrModuleRef, TokenKind::ValueRefOrIdent])?;

        self.end_temp_vec(Asn1Tag::Reference);
        Ok(())
    }

    /// Parse the references within an import statement
    pub(super) fn symbols_imported(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::SymbolsFromModuleList);

        loop {
            let tok = self.peek(&[
                TokenKind::TypeOrModuleRef,
                TokenKind::ValueRefOrIdent,
                TokenKind::SemiColon,
            ])?;
            if tok.kind == TokenKind::SemiColon {
                break;
            }

            self.symbols_from_module()?;
        }

        self.end_temp_vec(Asn1Tag::SymbolsFromModuleList);
        Ok(())
    }

    /// Parse a single section within an import statement
    fn symbols_from_module(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::SymbolsFromModule);

        self.symbol_list(SymbolListKind::Imports)?;
        self.next(&[TokenKind::KwFrom])?;
        self.global_module_reference()?;

        let tok = self.peek(&[
            TokenKind::KwWith,
            TokenKind::SemiColon,
            TokenKind::LeftCurly,
            TokenKind::ValueRefOrIdent,
            TokenKind::TypeOrModuleRef,
        ])?;
        if tok.kind == TokenKind::KwWith {
            self.selection_option()?;
        }

        self.end_temp_vec(Asn1Tag::SymbolsFromModule);
        Ok(())
    }

    /// Parse the part after the `FROM` keyword in an import statement
    ///
    /// The options are as follows:
    /// ```bnf
    /// FROM ModuleReference ;
    /// FROM ModuleReference {...} ;
    /// FROM ModuleReference (ValueReference | ModuleReference . ValueReference) ;
    /// FROM ModuleReference WITH ... ;
    /// FROM ModuleReference {...} WITH ... ;
    /// FROM ModuleReference (ValueReference | ModuleReference . ValueReference) WITH ... ;
    /// FROM ModuleReference a (, | FROM)
    /// FROM ModuleReference {...} a (, | FROM)
    /// FROM ModuleReference (ValueReference | ModuleReference . ValueReference) a (, | FROM)
    /// FROM ModuleReference WITH ... a (, | FROM)
    /// FROM ModuleReference {...} WITH ... a (, | FROM)
    /// FROM ModuleReference (ValueReference | ModuleReference . ValueReference) WITH ... a (, | FROM)
    /// ```
    /// The `a` is either a type or value reference that is part of the following
    /// symbol from module.  The semicolons and with sections are not part of this
    /// global module reference and will be parsed after this function returns.
    fn global_module_reference(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::GlobalModuleReference);

        self.next(&[TokenKind::TypeOrModuleRef])?;

        let tok = self.peek(&[
            TokenKind::SemiColon,
            TokenKind::LeftCurly,
            TokenKind::ValueRefOrIdent,
            TokenKind::TypeOrModuleRef,
            TokenKind::KwWith,
        ])?;

        if tok.kind == TokenKind::LeftCurly {
            self.object_identifier_value()?;
        } else if tok.kind == TokenKind::ValueRefOrIdent || tok.kind == TokenKind::TypeOrModuleRef {
            let tok = self.peek_n(&[
                &[TokenKind::ValueRefOrIdent, TokenKind::TypeOrModuleRef],
                &[
                    TokenKind::SemiColon,
                    TokenKind::Dot,
                    TokenKind::KwWith,
                    TokenKind::Comma,
                    TokenKind::KwFrom,
                    TokenKind::TypeOrModuleRef,
                    TokenKind::ValueRefOrIdent,
                ],
            ])?;

            if !matches!(tok.kind, TokenKind::Comma | TokenKind::KwFrom) {
                self.defined_value()?;
            }
        }

        self.end_temp_vec(Asn1Tag::GlobalModuleReference);
        Ok(())
    }

    /// Parse `with successors` or `with descendants`
    fn selection_option(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::SelectionOption);

        self.next(&[TokenKind::KwWith])?;
        self.next(&[TokenKind::CtxKwSuccessors, TokenKind::CtxKwDescendants])?;

        self.end_temp_vec(Asn1Tag::SelectionOption);
        Ok(())
    }
}*/
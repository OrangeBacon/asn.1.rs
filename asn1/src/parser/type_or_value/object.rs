//! object class definition

use crate::{cst::Asn1Tag, lexer::SquareBracketMode, token::TokenKind};

use super::{Parser, Result, TypeOrValue, TypeOrValueResult};

impl<'a> Parser<'a> {
    /// Parse an object class definition
    /// ```asn1
    /// ObjectClassDefinition ::=
    ///     CLASS
    ///     "{" FieldSpec "," + "}"
    ///     WithSyntaxSpec?
    /// ```
    pub(super) fn object_class(&mut self, expecting: TypeOrValue) -> Result {
        self.start_temp_vec(Asn1Tag::ObjectClass)?;

        self.next(&[TokenKind::KwClass])?;
        self.next(&[TokenKind::LeftCurly])?;
        self.field_spec_list()?;
        self.next(&[TokenKind::RightCurly])?;

        self.syntax_spec(expecting)?;

        self.end_temp_vec(Asn1Tag::ObjectClass);
        Ok(())
    }

    /// Parse a comma separated list of object class field specs
    fn field_spec_list(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::FieldSpecList)?;

        loop {
            self.field_spec()?;
            let tok = self.peek(&[TokenKind::RightCurly, TokenKind::Comma])?;
            if tok.kind == TokenKind::RightCurly {
                break;
            }
            self.next(&[TokenKind::Comma])?;
        }

        self.end_temp_vec(Asn1Tag::FieldSpecList);
        Ok(())
    }

    /// Parse a single object class field spec
    fn field_spec(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::FieldSpec)?;

        let tok = self.peek(&[TokenKind::TypeField, TokenKind::ValueField])?;
        if tok.kind == TokenKind::TypeField {
            self.type_field_spec()?;
        } else {
            self.value_field_spec()?;
        }

        self.end_temp_vec(Asn1Tag::FieldSpec);
        Ok(())
    }

    /// Parse a field spec starting with a type field reference
    fn type_field_spec(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::TypeFieldSpec)?;

        self.next(&[TokenKind::TypeField])?;

        let ty = self.type_or_value(TypeOrValue {
            alternative: &[
                TokenKind::KwOptional,
                TokenKind::KwDefault,
                TokenKind::Comma,
                TokenKind::RightCurly,
                TokenKind::TypeField,
                TokenKind::ValueField,
            ],
            subsequent: &[
                TokenKind::KwOptional,
                TokenKind::KwDefault,
                TokenKind::Comma,
                TokenKind::RightCurly,
            ],
        })?;
        if matches!(
            ty,
            TypeOrValueResult::Alternate(TokenKind::TypeField | TokenKind::ValueField)
        ) {
            self.next(&[TokenKind::TypeField, TokenKind::ValueField])?;
            self.field(&[
                TokenKind::KwOptional,
                TokenKind::KwDefault,
                TokenKind::Comma,
                TokenKind::RightCurly,
            ])?;
        }

        self.optionality_spec()?;

        self.end_temp_vec(Asn1Tag::TypeFieldSpec);
        Ok(())
    }

    /// Parse a field spec starting with a value field reference
    fn value_field_spec(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::ValueFieldSpec)?;

        self.next(&[TokenKind::ValueField])?;

        let subsequent = &[
            TokenKind::KwUnique,
            TokenKind::KwOptional,
            TokenKind::KwDefault,
            TokenKind::Comma,
            TokenKind::RightCurly,
        ];
        let ty = self.type_or_value(TypeOrValue {
            alternative: &[TokenKind::TypeField, TokenKind::ValueField],
            subsequent,
        })?;

        if ty == TypeOrValueResult::TypeOrValue {
            if self.peek(subsequent)?.kind == TokenKind::KwUnique {
                self.next(&[TokenKind::KwUnique])?;
            }
        } else {
            self.next(&[TokenKind::TypeField, TokenKind::ValueField])?;
            self.field(&[
                TokenKind::KwOptional,
                TokenKind::KwDefault,
                TokenKind::Comma,
                TokenKind::RightCurly,
            ])?;
        }

        self.optionality_spec()?;

        self.end_temp_vec(Asn1Tag::ValueFieldSpec);
        Ok(())
    }

    /// Optionally parse optional or default specifiers
    fn optionality_spec(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::OptionalitySpec)?;

        let tok = self.peek(&[
            TokenKind::KwOptional,
            TokenKind::KwDefault,
            TokenKind::Comma,
            TokenKind::RightCurly,
        ])?;

        if tok.kind == TokenKind::KwOptional {
            self.next(&[TokenKind::KwOptional])?;
        } else if tok.kind == TokenKind::KwDefault {
            self.next(&[TokenKind::KwDefault])?;

            self.type_or_value(TypeOrValue {
                alternative: &[],
                subsequent: &[TokenKind::Comma, TokenKind::RightCurly],
            })?;
        }

        self.end_temp_vec(Asn1Tag::OptionalitySpec);
        Ok(())
    }

    /// Parse a syntax specification for an object class
    fn syntax_spec(&mut self, expecting: TypeOrValue) -> Result {
        self.start_temp_vec(Asn1Tag::SyntaxSpec)?;

        let mut kind = expecting.subsequent.to_vec();
        kind.push(TokenKind::KwWith);
        let tok = self.peek(kind)?;
        if tok.kind != TokenKind::KwWith {
            return Ok(());
        }

        self.next(&[TokenKind::KwWith])?;
        self.next(&[TokenKind::KwSyntax])?;
        self.next(&[TokenKind::LeftCurly])?;

        self.lexer.set_square_bracket_mode(SquareBracketMode::Split);
        self.lexer.enable_keywords(false);

        self.syntax_spec_list(TokenKind::RightCurly)?;

        self.lexer.enable_keywords(true);
        self.lexer.set_square_bracket_mode(SquareBracketMode::Join);

        self.next(&[TokenKind::RightCurly])?;

        self.end_temp_vec(Asn1Tag::SyntaxSpec);
        Ok(())
    }

    /// Parse the contents of a syntax specification
    fn syntax_spec_list(&mut self, next: TokenKind) -> Result {
        self.start_temp_vec(Asn1Tag::SyntaxSpecList)?;

        loop {
            let tok = self.peek(
                [
                    TokenKind::TypeOrModuleRef,
                    TokenKind::Comma,
                    TokenKind::TypeField,
                    TokenKind::ValueField,
                    TokenKind::LeftSquare,
                    next,
                ]
                .to_vec(),
            )?;
            if tok.kind == next {
                break;
            } else if tok.kind != TokenKind::LeftSquare {
                self.next(&[])?;
                continue;
            } else {
                self.optional_syntax_spec()?;
            }
        }

        self.end_temp_vec(Asn1Tag::SyntaxSpecList);
        Ok(())
    }

    /// Parse an optional portion of a syntax specification
    fn optional_syntax_spec(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::OptionalSyntaxSpec)?;

        self.next(&[TokenKind::LeftSquare])?;
        self.syntax_spec_list(TokenKind::RightSquare)?;
        self.next(&[TokenKind::RightSquare])?;

        self.end_temp_vec(Asn1Tag::OptionalSyntaxSpec);
        Ok(())
    }
}

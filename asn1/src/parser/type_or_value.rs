mod composite_ty;
mod ty;
mod value;
mod object;

use crate::{
    cst::Asn1Tag,
    parser::{Parser, ParserError, Result},
    token::TokenKind,
};

/// Information to instruct how a type or a value should be parsed
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub(super) struct TypeOrValue<'a> {
    /// Alternative tokens that could appear instead of a type or value
    pub alternative: &'a [TokenKind],

    /// The tokens that are permissible after a type or value has finished parsing
    pub subsequent: &'a [TokenKind],
}

/// What was parsed by the type or value parser
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum TypeOrValueResult {
    /// A type or value declaration was successfully parsed
    TypeOrValue,

    /// No tokens were consumed, however the following token was peeked.  The kind
    /// of this token will be one of the kinds provided in the alternate token list.
    Alternate(TokenKind),
}

impl<'a> Parser<'a> {
    /// Parse either a type or a value declaration
    pub(super) fn type_or_value(&mut self, expecting: TypeOrValue) -> Result<TypeOrValueResult> {
        self.type_or_value_maybe_named(expecting, false)
    }

    /// Parse either a type or a value declaration, allowing named types.
    pub(super) fn type_or_value_named(
        &mut self,
        expecting: TypeOrValue,
    ) -> Result<TypeOrValueResult> {
        self.type_or_value_maybe_named(expecting, true)
    }

    /// Parse either a type or a value declaration
    fn type_or_value_maybe_named(
        &mut self,
        expecting: TypeOrValue,
        named: bool,
    ) -> Result<TypeOrValueResult> {
        // TODO type: constrained type

        let tok = self.peek(&[])?;

        if expecting.alternative.contains(&tok.kind) {
            return Ok(TypeOrValueResult::Alternate(tok.kind));
        }

        self.start_temp_vec(Asn1Tag::TypeOrValue)?;

        match tok.kind {
            // either
            TokenKind::TypeOrModuleRef => self.defined(expecting)?,
            TokenKind::ValueRefOrIdent => self.ident_type_value(expecting, named)?,

            // values
            TokenKind::Number | TokenKind::Hyphen => self.number_value()?,
            TokenKind::LeftCurly => self.braced_value()?,
            TokenKind::KwContaining => self.containing_value(expecting.subsequent)?,
            TokenKind::CString
            | TokenKind::KwTrue
            | TokenKind::KwFalse
            | TokenKind::BHString
            | TokenKind::KwPlusInfinity
            | TokenKind::KwNotANumber
            | TokenKind::KwMinusInfinity => {
                self.next(&[])?;
            }

            // types
            TokenKind::KwInteger => self.integer_type(expecting)?,
            TokenKind::KwEnumerated => self.enumerated_type(expecting)?,
            TokenKind::KwObject => self.object_identifier_type(expecting)?,
            TokenKind::KwBit => self.bit_string_type(expecting)?,
            TokenKind::KwOctet => self.octet_string_type(expecting)?,
            TokenKind::KwCharacter => self.character_string_type(expecting)?,
            TokenKind::KwInstance => self.instance_of_type(expecting)?,
            TokenKind::KwEmbedded => self.embedded_pdv_type(expecting)?,
            TokenKind::KwSequence => self.sequence_type(expecting)?,
            TokenKind::KwSet => self.set_type(expecting)?,
            TokenKind::KwChoice => self.choice_type(expecting)?,

            TokenKind::KwAbstractSyntax | TokenKind::KwTypeIdentifier => {
                self.object_fields(expecting)?
            }

            TokenKind::LeftSquare => self.prefix_type(expecting)?,

            TokenKind::KwBoolean
            | TokenKind::KwNull
            | TokenKind::KwOidIri
            | TokenKind::KwGeneralizedTime
            | TokenKind::KwUTCTime
            | TokenKind::KwObjectDescriptor
            | TokenKind::KwReal
            | TokenKind::KwRelativeOid
            | TokenKind::KwRelativeOidIri
            | TokenKind::KwExternal
            | TokenKind::KwTime
            | TokenKind::KwDate
            | TokenKind::KwTimeOfDay
            | TokenKind::KwDateTime
            | TokenKind::KwDuration
            | TokenKind::KwBmpString
            | TokenKind::KwGeneralString
            | TokenKind::KwGraphicString
            | TokenKind::KwIA5String
            | TokenKind::KwISO64String
            | TokenKind::KwNumericString
            | TokenKind::KwPrintableString
            | TokenKind::KwTeletexString
            | TokenKind::KwT61String
            | TokenKind::KwUniversalString
            | TokenKind::KwUTF8String
            | TokenKind::KwVideotexString
            | TokenKind::KwVisibleString => {
                self.next(&[])?;
                self.open_type_field_value(expecting)?;
            }

            // object definition - used in object class assignment.
            // ```
            // ObjectClass ::=
            //     DefinedObjectClass
            //   | ObjectClassDefinition
            //   | ParameterizedObjectClass
            // ```
            // the defined and parameterized options of this will be parsed as
            // type references and changed to object references later, so object
            // class definition is all that needs to be added.
            TokenKind::KwClass => self.object_class(expecting)?,

            _ => {
                return Err(ParserError::TypeValueError {
                    subsequent: expecting.subsequent.to_vec(),
                    alternative: expecting.alternative.to_vec(),
                    offset: tok.offset,
                    file: tok.file,
                });
            }
        }

        self.end_temp_vec(Asn1Tag::TypeOrValue);
        Ok(TypeOrValueResult::TypeOrValue)
    }

    /// Parse a reference to a previously defined type or value.
    /// ```bnf
    /// DefinedType ::=
    ///     ExternalTypeReference
    ///   | type_reference
    ///   | ParameterizedType
    ///   | ParameterizedValueSetType
    ///
    /// DefinedValue ::=
    ///     ExternalValueReference
    ///   | value_reference
    ///   | ParameterizedValue
    /// ```
    /// This function only works where the first token is a type reference,
    /// therefore cannot take into account a value reference or a parametrised
    /// value reference.
    fn defined(&mut self, expecting: TypeOrValue) -> Result {
        self.start_temp_vec(Asn1Tag::Defined)?;

        self.next(&[TokenKind::TypeOrModuleRef])?;

        let mut kind = expecting.subsequent.to_vec();
        kind.push(TokenKind::Dot);
        kind.push(TokenKind::LeftCurly);
        kind.push(TokenKind::Colon);
        let tok = self.peek(kind)?;

        if tok.kind == TokenKind::Dot {
            self.next(&[TokenKind::Dot])?;

            let tok = self.peek(&[
                TokenKind::ValueRefOrIdent,
                TokenKind::TypeOrModuleRef,
                TokenKind::Field,
            ])?;
            if tok.kind != TokenKind::Field {
                self.next(&[])?;
            }
        }

        let mut kind = expecting.subsequent.to_vec();
        kind.push(TokenKind::Dot);
        kind.push(TokenKind::LeftCurly);
        kind.push(TokenKind::Colon);
        let tok = self.peek(kind)?;

        if tok.kind == TokenKind::LeftCurly {
            self.actual_parameter_list()?;
        }

        let mut kind = expecting.subsequent.to_vec();
        kind.push(TokenKind::Dot);
        kind.push(TokenKind::Colon);

        let tok = self.peek(kind)?;
        if tok.kind == TokenKind::Dot {
            self.field(expecting.subsequent)?;
        }

        self.end_temp_vec(Asn1Tag::Defined);

        self.open_type_field_value(expecting)?;

        Ok(())
    }

    /// Parse a type or a value that begins with an identifier token.
    /// ```asn1
    /// DefinedValue ::= value_reference | ParameterizedValue
    /// IntegerValue ::= identifier
    /// EnumeratedValue ::= identifier
    /// ChoiceValue ::= identifier ':' Value
    /// SelectionType ::= identifier "<" Type
    /// ```
    /// `IntegerValue`, `EnumeratedValue` and the first option of `DefinedValue`
    /// are all identical to the parser so will be distinguished later.
    ///
    /// If `named` is true, then allows parsing named values.
    fn ident_type_value(&mut self, expecting: TypeOrValue, named: bool) -> Result {
        self.start_temp_vec(Asn1Tag::Defined)?;

        self.next(&[TokenKind::ValueRefOrIdent])?;

        let mut kind = expecting.subsequent.to_vec();
        kind.push(TokenKind::Colon);
        kind.push(TokenKind::LeftCurly);
        kind.push(TokenKind::Less);
        kind.push(TokenKind::Dot);

        let tok = if named {
            let ty = self.type_or_value(TypeOrValue {
                alternative: &[
                    TokenKind::Colon,
                    TokenKind::LeftCurly,
                    TokenKind::Less,
                    TokenKind::Dot,
                ],
                subsequent: expecting.subsequent,
            })?;

            match ty {
                TypeOrValueResult::TypeOrValue => {
                    self.end_temp_vec(Asn1Tag::Defined);
                    return Ok(());
                }
                TypeOrValueResult::Alternate(tok) => tok,
            }
        } else {
            self.peek(kind)?.kind
        };

        match tok {
            TokenKind::LeftCurly => {
                self.actual_parameter_list()?;
                let mut kind = expecting.subsequent.to_vec();
                kind.push(TokenKind::Dot);
                if self.peek(kind)?.kind == TokenKind::Dot {
                    self.field(expecting.subsequent)?;
                }
            }
            TokenKind::Less => self.selection_type(expecting.subsequent)?,
            TokenKind::Colon => self.choice_value(expecting.subsequent)?,
            TokenKind::Dot => self.field(expecting.subsequent)?,
            _ => (),
        }

        self.end_temp_vec(Asn1Tag::Defined);

        Ok(())
    }

    /// Parse object field names
    /// `("." Field)+`
    fn field(&mut self, subsequent: &[TokenKind]) -> Result {
        self.start_temp_vec(Asn1Tag::FieldNames)?;

        loop {
            self.next(&[TokenKind::Dot])?;
            self.next(&[TokenKind::Field])?;

            let mut kind = subsequent.to_vec();
            kind.push(TokenKind::Dot);
            if self.peek(kind)?.kind != TokenKind::Dot {
                break;
            }
        }

        self.end_temp_vec(Asn1Tag::FieldNames);
        Ok(())
    }
}

impl TypeOrValueResult {
    /// Is this result specifying an an alternate token of kind assignment
    pub fn is_assign(self) -> bool {
        self == TypeOrValueResult::Alternate(TokenKind::Assignment)
    }
}

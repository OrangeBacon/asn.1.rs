mod ty;
mod value;

use std::{collections::HashSet, sync::OnceLock};

use crate::{cst::Asn1Tag, token::TokenKind};

use super::{Parser, Result};

/// Information to instruct how a type or a value should be parsed
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub(super) struct TypeOrValue<'a> {
    /// If true, accept a type declaration
    pub is_type: bool,

    /// If true, accept a value declaration
    pub is_value: bool,

    /// If true accept a defined value
    pub is_defined_value: bool,

    /// Alternative tokens that could appear instead of a type or value
    pub alternative: &'a [TokenKind],

    /// The tokens that are permissible after a type or value has finished parsing
    pub subsequent: &'a [TokenKind],

    /// The tokens that are permissible after a defined value has finished parsing.
    /// This overrides `subsequent` in the case that is_defined_value is true
    /// and a defined value is being matched.
    pub defined_subsequent: &'a [TokenKind],
}

/// What was parsed by the type or value parser
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum TypeOrValueResult {
    /// A type declaration was successfully parsed
    Type,

    /// A value declaration was successfully parsed
    Value,

    /// Parsed a declaration that could be either a type or a value
    Ambiguous,

    /// No tokens were consumed, however the following token was peeked.  The kind
    /// of this token will be one of the kinds provided in the alternate token list.
    Alternate(TokenKind),
}

const TYPE_START: &[TokenKind] = &[
    // builtin types
    TokenKind::KwBoolean,
    TokenKind::KwNull,
    TokenKind::KwOidIri,
    TokenKind::KwInteger,
    TokenKind::KwEnumerated,
    TokenKind::KwObject,
    // reference type
    TokenKind::TypeOrModuleRef,
    TokenKind::ValueRefOrIdent,
    // useful type
    TokenKind::KwGeneralizedTime,
    TokenKind::KwUTCTime,
    TokenKind::KwObjectDescriptor,
];

const VALUE_START: &[TokenKind] = &[
    TokenKind::CString,
    TokenKind::KwTrue,
    TokenKind::KwFalse,
    TokenKind::KwNull,
    TokenKind::Number,
    TokenKind::Hyphen,
    TokenKind::LeftCurly,
    // referenced value
    TokenKind::TypeOrModuleRef,
    TokenKind::ValueRefOrIdent,
];

impl<'a> Parser<'a> {
    /// Parse either a type or a value declaration
    pub(super) fn type_or_value(&mut self, expecting: TypeOrValue) -> Result<TypeOrValueResult> {
        // TODO value: bit string, character string, instance of,
        // octet string, real, time, value from object, object class field value

        // TODO type: Bit string, character string, choice, date, date time, duration
        // embedded pdv, external, instance of, object class field,
        // octet string, real, relative iri, relative oid, sequence,
        // sequence of, set, set of, prefixed, time, time of day, type from object,
        // value set from objects, constrained type

        let kind = match (expecting.is_type, expecting.is_value) {
            (true, true) => both_start(),
            (true, false) => TYPE_START,
            (false, true) => VALUE_START,
            (false, false) => {
                if expecting.is_defined_value {
                    &[TokenKind::TypeOrModuleRef, TokenKind::ValueRefOrIdent][..]
                } else {
                    &[]
                }
            }
        };

        let mut kind = kind.to_owned();
        kind.extend(expecting.alternative);

        let tok = self.peek(kind)?;

        // TODO: distinguish between integer ident and selection ident
        // TokenKind::ValueRefOrIdent => self.selection_type(expecting.subsequent)?,

        Ok(match tok.kind {
            // either
            TokenKind::KwNull if expecting.is_type || expecting.is_value => {
                let tag = if expecting.is_type && expecting.is_value {
                    Asn1Tag::TypeOrValue
                } else if expecting.is_type {
                    Asn1Tag::Type
                } else {
                    Asn1Tag::Value
                };
                self.start_temp_vec(tag)?;
                self.next(&[TokenKind::KwNull])?;
                self.end_temp_vec(tag);
                TypeOrValueResult::Ambiguous
            }
            TokenKind::TypeOrModuleRef
                if expecting.is_type || expecting.is_value || expecting.is_defined_value =>
            {
                self.start_temp_vec(Asn1Tag::TypeOrValue)?;
                let res = self.defined(expecting)?;
                self.end_temp_vec(Asn1Tag::TypeOrValue);
                res
            }
            TokenKind::ValueRefOrIdent
                if expecting.is_type || expecting.is_value || expecting.is_defined_value =>
            {
                self.start_temp_vec(Asn1Tag::TypeOrValue)?;
                let res = self.ident_type_value(expecting)?;
                self.end_temp_vec(Asn1Tag::TypeOrValue);
                res
            }

            // values
            TokenKind::Number | TokenKind::Hyphen if expecting.is_value => {
                self.start_temp_vec(Asn1Tag::Value)?;
                self.integer_value()?;
                self.end_temp_vec(Asn1Tag::Value);
                TypeOrValueResult::Value
            }
            TokenKind::LeftCurly if expecting.is_value => {
                self.start_temp_vec(Asn1Tag::Value)?;
                self.braced_value()?;
                self.end_temp_vec(Asn1Tag::Value);
                TypeOrValueResult::Value
            }
            TokenKind::CString if expecting.is_value => {
                self.start_temp_vec(Asn1Tag::Value)?;
                self.next(&[TokenKind::CString])?;
                self.end_temp_vec(Asn1Tag::Value);
                TypeOrValueResult::Value
            }
            TokenKind::KwTrue | TokenKind::KwFalse if expecting.is_value => {
                self.start_temp_vec(Asn1Tag::Value)?;
                self.next(&[TokenKind::KwTrue, TokenKind::KwFalse])?;
                self.end_temp_vec(Asn1Tag::Value);
                TypeOrValueResult::Value
            }

            // types
            TokenKind::KwInteger if expecting.is_type => {
                self.start_temp_vec(Asn1Tag::Type)?;
                self.integer_type(expecting.subsequent)?;
                self.end_temp_vec(Asn1Tag::Type);
                TypeOrValueResult::Type
            }
            TokenKind::KwEnumerated if expecting.is_type => {
                self.start_temp_vec(Asn1Tag::Type)?;
                self.enumerated_type()?;
                self.end_temp_vec(Asn1Tag::Type);
                TypeOrValueResult::Type
            }
            TokenKind::KwObject if expecting.is_type => {
                self.start_temp_vec(Asn1Tag::Type)?;
                self.object_identifier_type()?;
                self.end_temp_vec(Asn1Tag::Type);
                TypeOrValueResult::Type
            }

            TokenKind::KwBoolean
            | TokenKind::KwNull
            | TokenKind::KwOidIri
            | TokenKind::KwGeneralizedTime
            | TokenKind::KwUTCTime
            | TokenKind::KwObjectDescriptor
                if expecting.is_type =>
            {
                self.start_temp_vec(Asn1Tag::Type)?;
                self.next(&[
                    TokenKind::KwBoolean,
                    TokenKind::KwNull,
                    TokenKind::KwOidIri,
                    TokenKind::KwGeneralizedTime,
                    TokenKind::KwUTCTime,
                    TokenKind::KwObjectDescriptor,
                ])?;
                self.end_temp_vec(Asn1Tag::Type);
                TypeOrValueResult::Type
            }
            _ => {
                self.peek(expecting.alternative.to_owned())?;
                TypeOrValueResult::Alternate(tok.kind)
            }
        })
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
    fn defined(&mut self, expecting: TypeOrValue) -> Result<TypeOrValueResult> {
        self.start_temp_vec(Asn1Tag::Defined)?;

        self.next(&[TokenKind::TypeOrModuleRef])?;

        let mut ret = TypeOrValueResult::Type;

        let tok = if expecting.is_type {
            let mut kind = expecting.subsequent.to_vec();
            kind.push(TokenKind::Dot);
            kind.push(TokenKind::LeftCurly);
            self.peek(kind)?
        } else {
            self.peek(&[TokenKind::Dot])?
        };

        if tok.kind == TokenKind::Dot {
            self.next(&[TokenKind::Dot])?;

            let kind = match (
                expecting.is_type,
                expecting.is_value | expecting.is_defined_value,
            ) {
                (true, true) => &[TokenKind::ValueRefOrIdent, TokenKind::TypeOrModuleRef][..],
                (true, false) => &[TokenKind::TypeOrModuleRef],
                (false, true) => &[TokenKind::ValueRefOrIdent],
                (false, false) => &[],
            };

            self.next(kind)?;

            ret = TypeOrValueResult::Value;
        }

        let mut kind = if ret == TypeOrValueResult::Value && expecting.is_defined_value {
            expecting.defined_subsequent.to_vec()
        } else {
            expecting.subsequent.to_vec()
        };
        kind.push(TokenKind::LeftCurly);

        let tok = self.peek(kind)?;
        if tok.kind == TokenKind::LeftCurly {
            self.actual_parameter_list()?;
        }

        self.end_temp_vec(Asn1Tag::Defined);
        Ok(ret)
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
    fn ident_type_value(&mut self, expecting: TypeOrValue) -> Result<TypeOrValueResult> {
        self.next(&[TokenKind::ValueRefOrIdent])?;

        let mut kind = if expecting.is_defined_value {
            expecting.defined_subsequent.to_vec()
        } else {
            expecting.subsequent.to_vec()
        };

        if expecting.is_value {
            kind.push(TokenKind::Colon);
        }
        if expecting.is_value || expecting.is_defined_value {
            kind.push(TokenKind::LeftCurly);
        }
        if expecting.is_type {
            kind.push(TokenKind::Less);
        }

        let tok = self.peek(kind)?;

        if tok.kind == TokenKind::LeftCurly {
            self.actual_parameter_list()?;
        } else if tok.kind == TokenKind::Less {
            self.selection_type(expecting.subsequent)?;
        } else if tok.kind == TokenKind::Colon {
            self.choice_value(expecting.subsequent)?;
        }

        Ok(TypeOrValueResult::Ambiguous)
    }
}

impl TypeOrValueResult {
    /// Is this result specifying an an alternate token of kind assignment
    pub fn is_assign(self) -> bool {
        self == TypeOrValueResult::Alternate(TokenKind::Assignment)
    }
}

/// Concatenate the type and value start arrays at runtime as const array concat
/// is not possible without transmute fuckery i'd rather avoid, even if it works.
fn both_start() -> &'static [TokenKind] {
    static BOTH_START: OnceLock<Vec<TokenKind>> = OnceLock::new();
    BOTH_START.get_or_init(|| {
        TYPE_START
            .iter()
            .chain(VALUE_START)
            .copied()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    })
}

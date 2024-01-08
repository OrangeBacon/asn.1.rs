mod ty;
mod value;

use std::sync::OnceLock;

use crate::{cst::Asn1Tag, token::TokenKind};

use super::{Parser, Result};

/// Information to instruct how a type or a value should be parsed
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub(super) struct TypeOrValue<'a> {
    /// If true, accept a type declaration
    pub is_type: bool,

    /// If true, accept a value declaration
    pub is_value: bool,

    /// Alternative tokens that could appear instead of a type or value
    pub alternative: &'a [TokenKind],

    /// The tokens that are permissible after a type or value has finished parsing
    pub subsequent: &'a [TokenKind],
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
    TokenKind::DoubleQuote,
    TokenKind::KwTrue,
    TokenKind::KwFalse,
    TokenKind::KwNull,
    TokenKind::Number,
    TokenKind::Hyphen,
    TokenKind::ValueRefOrIdent,
    TokenKind::LeftCurly,
];

impl<'a> Parser<'a> {
    /// Parse either a type or a value declaration
    pub(super) fn type_or_value(&mut self, expecting: TypeOrValue) -> Result<TypeOrValueResult> {
        // TODO value: bit string, character string, choice, embedded pdv, enumerated,
        // external, instance of, integer, object identifier, octet string, real
        // relative iri, relative oid, sequence, sequence of, set, set of, prefixed,
        // time, referenced value, object class field value

        // TODO type: Bit string, character string, choice, date, date time, duration
        // embedded pdv, external, instance of, integer, object class field,
        // octet string, real, relative iri, relative oid, sequence,
        // sequence of, set, set of, prefixed, time, time of day, type from object,
        // value set from objects, constrained type, parameterized type

        // TODO: remove all calls to leak

        let kind = match (expecting.is_type, expecting.is_value) {
            (true, true) => both_start(),
            (true, false) => TYPE_START,
            (false, true) => VALUE_START,

            // should not reach this case as this function should never be called
            // to parse neither a type nor a value, so pick a default value
            (false, false) => &[],
        };

        let ambiguous_tag = match (expecting.is_type, expecting.is_value) {
            (true, true) => Asn1Tag::TypeOrValue,
            (true, false) => Asn1Tag::Type,
            (false, true) => Asn1Tag::Value,

            // should not reach this case as this function should never be called
            // to parse neither a type nor a value, so pick a default value
            (false, false) => Asn1Tag::TypeOrValue,
        };

        let mut kind = kind.to_owned();
        kind.extend(expecting.alternative);

        let tok = self.peek(kind.leak())?;

        let parse_kind;

        // TODO: distinguish between integer ident and selection ident
        // TokenKind::ValueRefOrIdent => self.selection_type(expecting.subsequent)?,

        match tok.kind {
            // either
            TokenKind::KwNull if expecting.is_type || expecting.is_value => {
                self.start_temp_vec(ambiguous_tag);
                parse_kind = TypeOrValueResult::Ambiguous;
                self.next(&[TokenKind::KwNull])?;
            }

            // values
            TokenKind::Number | TokenKind::Hyphen | TokenKind::ValueRefOrIdent
                if expecting.is_value =>
            {
                parse_kind = self.start_value();
                self.integer_value()?
            }
            TokenKind::LeftCurly if expecting.is_value => {
                parse_kind = self.start_value();
                self.object_identifier_value()?;
            }
            TokenKind::DoubleQuote if expecting.is_value => {
                parse_kind = self.start_value();
                self.iri_value()?;
            }
            TokenKind::KwTrue | TokenKind::KwFalse if expecting.is_value => {
                parse_kind = self.start_value();
                self.next(Vec::from([tok.kind]).leak())?;
            }

            // types
            TokenKind::KwInteger if expecting.is_type => {
                parse_kind = self.start_type();
                self.integer_type()?;
            }
            TokenKind::KwEnumerated if expecting.is_type => {
                parse_kind = self.start_type();
                self.enumerated_type()?;
            }
            TokenKind::KwObject if expecting.is_type => {
                parse_kind = self.start_type();
                self.object_identifier_type()?
            }
            TokenKind::TypeOrModuleRef if expecting.is_type => {
                parse_kind = self.start_type();
                self.defined_type()?
            }

            TokenKind::KwBoolean
            | TokenKind::KwNull
            | TokenKind::KwOidIri
            | TokenKind::KwGeneralizedTime
            | TokenKind::KwUTCTime
            | TokenKind::KwObjectDescriptor
                if expecting.is_type =>
            {
                parse_kind = self.start_type();
                self.next(Vec::from([tok.kind]).leak())?;
            }
            _ => {
                self.peek(expecting.alternative.to_owned().leak())?;
                return Ok(TypeOrValueResult::Alternate(tok.kind));
            }
        }

        self.end_temp_vec(match parse_kind {
            TypeOrValueResult::Type => Asn1Tag::Type,
            TypeOrValueResult::Value => Asn1Tag::Value,
            TypeOrValueResult::Ambiguous | TypeOrValueResult::Alternate(_) => ambiguous_tag,
        });

        Ok(parse_kind)
    }

    /// Start a temporary vector for a type declaration and return result::type
    fn start_type(&mut self) -> TypeOrValueResult {
        self.start_temp_vec(Asn1Tag::Type);
        TypeOrValueResult::Type
    }

    /// Start a temporary vector for a value declaration and return result::value
    fn start_value(&mut self) -> TypeOrValueResult {
        self.start_temp_vec(Asn1Tag::Value);
        TypeOrValueResult::Value
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
    BOTH_START.get_or_init(|| TYPE_START.iter().chain(VALUE_START).copied().collect())
}

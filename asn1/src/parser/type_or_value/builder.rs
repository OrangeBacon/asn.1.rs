use crate::{
    parser::{Parser, Result},
    token::TokenKind,
    util::CowVec,
};

use super::TypeOrValueResult;

/// Information to instruct how a type or a value should be parsed
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct TypeOrValue<Vec = ()> {
    /// If true, accept a type declaration
    pub(super) is_type: bool,

    /// If true, accept a value declaration
    pub(super) is_value: bool,

    /// If true accept a defined value
    pub(super) is_defined_value: bool,

    /// Alternative tokens that could appear instead of a type or value
    pub(super) alternative: Vec,

    /// The tokens that are permissible after a type or value has finished parsing
    pub(super) subsequent: Vec,

    /// The tokens that are permissible after a defined value has finished parsing.
    /// This overrides `subsequent` in the case that a defined value is being matched.
    pub(super) defined_subsequent: Vec,
}

pub type TypeOrValueBuilder = TypeOrValue<CowVec<TokenKind>>;
pub type TypeOrValueRef<'a> = TypeOrValue<&'a [TokenKind]>;
pub type TypeOrValueOwned = TypeOrValue<Vec<TokenKind>>;

impl TypeOrValue<()> {
    /// Build a type or value parser
    pub fn builder() -> TypeOrValueBuilder {
        TypeOrValue::default()
    }
}

impl TypeOrValueBuilder {
    /// Allow types to be parsed with the given token kinds after the type
    pub fn ty(self, ty: impl Into<CowVec<TokenKind>>) -> Self {
        Self {
            is_type: true,
            subsequent: self.subsequent.append(ty),
            ..self
        }
    }

    /// Allow values to be parsed with the given token kinds after the type
    pub fn value(self, subsequent: impl Into<CowVec<TokenKind>>) -> Self {
        let val = self.subsequent.append(subsequent);

        Self {
            is_value: true,
            is_defined_value: true,
            subsequent: val.clone(),
            defined_subsequent: val,
            ..self
        }
    }

    /// Allow alternate tokens instead
    pub fn alternate(self, alternate: impl Into<CowVec<TokenKind>>) -> Self {
        Self {
            alternative: self.alternative.append(alternate),
            ..self
        }
    }

    /// Run a parser using this builder
    pub(in crate::parser) fn parse(&self, parser: &mut Parser) -> Result<TypeOrValueResult> {
        parser.type_or_value(TypeOrValue {
            is_type: self.is_type,
            is_value: self.is_value,
            is_defined_value: self.is_defined_value,
            alternative: &self.alternative,
            subsequent: &self.subsequent,
            defined_subsequent: &self.defined_subsequent,
        })
    }
}

impl TypeOrValueRef<'_> {
    pub fn to_owned(self) -> TypeOrValueOwned {
        TypeOrValue {
            is_type: self.is_type,
            is_value: self.is_value,
            is_defined_value: self.is_defined_value,
            alternative: self.alternative.to_vec(),
            subsequent: self.subsequent.to_vec(),
            defined_subsequent: self.defined_subsequent.to_vec(),
        }
    }
}

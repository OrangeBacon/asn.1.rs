use crate::{
    analysis::AnalysisContext,
    cst::{Asn1Tag, CstIter},
    diagnostic::Result,
    token::TokenKind,
};

use super::{module::AssignmentKind, WithId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Boolean,
    Null,
    OidIri,
    GeneralizedTime,
    UTCTime,
    ObjectDescriptor,
    Real,
    RelativeOid,
    RelativeOidIri,
    External,
    Time,
    Date,
    TimeOfDay,
    DateTime,
    Duration,
    BmpString,
    GeneralString,
    GraphicString,
    IA5String,
    ISO64String,
    NumericString,
    PrintableString,
    TeletexString,
    T61String,
    UniversalString,
    UTF8String,
    VideotexString,
    VisibleString,
}

impl AnalysisContext<'_> {
    /// Parse a type assignment
    pub(super) fn type_assignment(
        &self,
        iter: &mut CstIter,
    ) -> Result<(WithId<Type>, AssignmentKind)> {
        let mut inner = self.tree(iter.next(), Asn1Tag::TypeAssignment)?;
        iter.assert_empty()?;

        self.token(inner.next(), TokenKind::Assignment)?;
        let mut ty = self.tree(inner.next(), Asn1Tag::TypeOrValue)?;
        inner.assert_empty()?;

        let ret = self.type_or_value(&mut ty)?;

        Ok((ret, AssignmentKind::TypeAssignment))
    }

    /// Parse either a type or value
    fn type_or_value(&self, iter: &mut CstIter) -> Result<WithId<Type>> {
        let first = self.token(iter.next(), &[])?;

        let result = match first.kind {
            TokenKind::KwBoolean => Type::Boolean,
            TokenKind::KwNull => Type::Null,
            TokenKind::KwOidIri => Type::OidIri,
            TokenKind::KwGeneralizedTime => Type::GeneralizedTime,
            TokenKind::KwUTCTime => Type::UTCTime,
            TokenKind::KwObjectDescriptor => Type::ObjectDescriptor,
            TokenKind::KwReal => Type::Real,
            TokenKind::KwRelativeOid => Type::RelativeOid,
            TokenKind::KwRelativeOidIri => Type::RelativeOidIri,
            TokenKind::KwExternal => Type::External,
            TokenKind::KwTime => Type::Time,
            TokenKind::KwDate => Type::Date,
            TokenKind::KwTimeOfDay => Type::TimeOfDay,
            TokenKind::KwDateTime => Type::DateTime,
            TokenKind::KwDuration => Type::Duration,
            TokenKind::KwBmpString => Type::BmpString,
            TokenKind::KwGeneralString => Type::GeneralString,
            TokenKind::KwGraphicString => Type::GraphicString,
            TokenKind::KwIA5String => Type::IA5String,
            TokenKind::KwISO64String => Type::ISO64String,
            TokenKind::KwNumericString => Type::NumericString,
            TokenKind::KwPrintableString => Type::PrintableString,
            TokenKind::KwTeletexString => Type::TeletexString,
            TokenKind::KwT61String => Type::T61String,
            TokenKind::KwUniversalString => Type::UniversalString,
            TokenKind::KwUTF8String => Type::UTF8String,
            TokenKind::KwVideotexString => Type::VideotexString,
            TokenKind::KwVisibleString => Type::VisibleString,
            _ => todo!(),
        };

        Ok(WithId {
            value: result,
            id: iter.node,
        })
    }
}

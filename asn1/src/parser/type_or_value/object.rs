//! object class definition

use crate::{cst::Asn1Tag, token::TokenKind};

use super::{Parser, Result, TypeOrValue};

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
        self.next(&[TokenKind::RightCurly])?;

        self.end_temp_vec(Asn1Tag::ObjectClass);
        Ok(())
    }
}

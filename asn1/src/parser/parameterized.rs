use crate::{cst::Asn1Tag, token::TokenKind};

use super::{Parser, Result};

impl<'a> Parser<'a> {
    /// Parse the parameter list for a parameterized type
    /// ```bnf
    /// ActualParameterList ::= "{" ActualParameter ("," ActualParameter)* "}"
    /// ```
    pub(super) fn actual_parameter_list(&mut self) -> Result {
        self.start_temp_vec(Asn1Tag::ActualParameterList)?;

        self.next(&[TokenKind::LeftCurly])?;

        // TODO: contents

        self.next(&[TokenKind::RightCurly])?;

        self.end_temp_vec(Asn1Tag::ActualParameterList);
        Ok(())
    }
}

use crate::{
    analysis::{Analysis, Result},
    cst::{Asn1Tag, AsnNodeId},
    token::{Token, TokenKind},
};

/// A group of ASN.1 assignments and settings.
#[derive(Debug, Clone, Copy)]
pub struct ModuleDefinition {
    /// Identifier for the module
    pub name: Token,
}

impl Analysis<'_> {
    /// Try to get the ast for a module
    pub fn module_ast(&self, node: AsnNodeId) -> Result<ModuleDefinition> {
        let mut iter = self.tree(node, Asn1Tag::ModuleDefinition)?;

        let mut name = self.tree(iter.next(), Asn1Tag::ModuleIdentifier)?;
        let name = self.token(name.next(), TokenKind::TypeOrModuleRef)?;

        Ok(ModuleDefinition { name })
    }
}

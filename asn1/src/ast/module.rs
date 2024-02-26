use crate::{
    analysis::context::AnalysisContext,
    cst::{Asn1Tag, AsnNodeId, CstIter},
    token::{Token, TokenKind},
};

use super::error::Result;

/// A group of ASN.1 assignments and settings.
#[derive(Debug, Clone, Copy)]
pub struct ModuleDefinition<'a> {
    /// Identifier for the module
    pub name: &'a str,

    /// Name of the default encoding
    pub encoding_reference: Option<Token>,

    /// How automatic tagging should be performed
    pub tag_default: TagDefault,

    /// Is extensibility implied in this module
    pub extensibility: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum TagDefault {
    None,
    Automatic,
    Implicit,
    Explicit,
}

impl AnalysisContext<'_> {
    /// Try to get the ast for a module
    pub fn module_ast(&self, node: AsnNodeId) -> Result<ModuleDefinition> {
        let mut iter = self.tree(node, Asn1Tag::ModuleDefinition)?;

        let mut name_iter = self.tree(iter.next(), Asn1Tag::ModuleIdentifier)?;
        let name = self.token(name_iter.next(), TokenKind::TypeOrModuleRef)?;
        let name = self.token_value(name);
        name_iter.assert_empty()?;

        self.token(iter.next(), TokenKind::KwDefinitions)?;

        let mut defaults_iter = self.tree(iter.next(), Asn1Tag::ModuleDefaults)?;
        let encoding_reference = self.encoding_reference(&mut defaults_iter)?;
        let tag_default = self.tag_default(&mut defaults_iter)?;
        let extensibility = self.extensibility(&mut defaults_iter)?;
        defaults_iter.assert_empty()?;

        self.token(iter.next(), TokenKind::Assignment)?;
        self.token(iter.next(), TokenKind::KwBegin)?;

        Ok(ModuleDefinition {
            name,
            encoding_reference,
            tag_default,
            extensibility,
        })
    }

    /// Interpret the encoding reference cst node
    fn encoding_reference(&self, iter: &mut CstIter) -> Result<Option<Token>> {
        let mut iter = self.tree(iter.next(), Asn1Tag::EncodingReferenceDefault)?;

        if iter.peek().is_some() {
            let encoding = self.token(iter.next(), TokenKind::TypeOrModuleRef)?;
            self.token(iter.next(), TokenKind::KwInstructions)?;
            iter.assert_empty()?;
            Ok(Some(encoding))
        } else {
            Ok(None)
        }
    }

    /// Interpret the tag default cst node
    fn tag_default(&self, iter: &mut CstIter) -> Result<TagDefault> {
        let mut iter = self.tree(iter.next(), Asn1Tag::TagDefault)?;
        if iter.peek().is_none() {
            return Ok(TagDefault::None);
        }
        let tok = self.token(
            iter.next(),
            &[
                TokenKind::KwImplicit,
                TokenKind::KwExplicit,
                TokenKind::KwAutomatic,
            ],
        )?;
        self.token(iter.next(), TokenKind::KwTags)?;
        iter.assert_empty()?;

        Ok(match tok.kind {
            TokenKind::KwImplicit => TagDefault::Implicit,
            TokenKind::KwExplicit => TagDefault::Explicit,
            _ => TagDefault::Automatic,
        })
    }

    /// Interpret the extension default cst node
    fn extensibility(&self, iter: &mut CstIter) -> Result<bool> {
        let mut iter = self.tree(iter.next(), Asn1Tag::ExtensionDefault)?;
        if iter.peek().is_none() {
            return Ok(false);
        }
        self.token(iter.next(), TokenKind::KwExtensibility)?;
        self.token(iter.next(), TokenKind::KwImplied)?;
        iter.assert_empty()?;

        Ok(true)
    }
}

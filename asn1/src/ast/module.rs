use crate::{
    analysis::{AnalysisContext, Iri, Oid, OidComponent},
    cst::{Asn1Tag, AsnNodeId, CstIter},
    token::{Token, TokenKind},
};

use super::{error::Result, AstError};

/// A group of ASN.1 assignments and settings.
#[derive(Debug, Clone)]
pub struct ModuleDefinition<'a> {
    /// Identifier for the module
    pub name: ModuleIdentifier<'a>,

    /// Name of the default encoding
    pub encoding_reference: Option<Token>,

    /// How automatic tagging should be performed
    pub tag_default: TagDefault,

    /// Is extensibility implied in this module
    pub extensibility: bool,
}

#[derive(Debug, Clone)]
pub struct ModuleIdentifier<'a> {
    /// Identifier for the module
    pub name: &'a str,

    /// The module's object identifier
    pub oid: Option<Oid>,

    /// The module's internationalized resource identifier value
    pub iri: Option<Iri>,
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
    pub(crate) fn module_ast(&self, node: AsnNodeId) -> Result<ModuleDefinition> {
        let mut iter = self.tree(node, Asn1Tag::ModuleDefinition)?;

        let name = self.module_identifier(&mut iter)?;

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

    /// Interpret a module identifier
    fn module_identifier(&self, iter: &mut CstIter) -> Result<ModuleIdentifier> {
        let mut iter = self.tree(iter.next(), Asn1Tag::ModuleIdentifier)?;
        let name = self.token(iter.next(), TokenKind::TypeOrModuleRef)?;
        let name = self.token_value(name);

        let oid = if iter.peek().is_some() {
            Some(self.module_oid(&mut iter)?)
        } else {
            None
        };
        let iri = if let Some(node) = iter.peek() {
            let tok = self.token(iter.next(), TokenKind::CString)?;
            let iri = Iri::from_str(self.token_value(tok))
                .map_err(|err| AstError::IriParseError { err, node })?;
            Some(iri)
        } else {
            None
        };

        iter.assert_empty()?;

        Ok(ModuleIdentifier { name, oid, iri })
    }

    /// Interpret a module identifier's oid
    fn module_oid(&self, iter: &mut CstIter) -> Result<Oid> {
        let mut iter = self.tree(iter.next(), Asn1Tag::DefinitiveOID)?;
        self.token(iter.next(), TokenKind::LeftCurly)?;

        let mut oid = vec![];
        while let Ok(mut comp) = self.tree(iter.peek(), Asn1Tag::DefinitiveOIDComponent) {
            iter.next();
            let comp = self.module_oid_component(&mut comp)?;
            oid.push(comp);
        }

        self.token(iter.next(), TokenKind::RightCurly)?;

        Ok(Oid { components: oid })
    }

    /// Interpret the inside of a definitive oid component
    fn module_oid_component(&self, iter: &mut CstIter) -> Result<OidComponent> {
        let mut comp = OidComponent {
            label: None,
            number: None,
        };
        let tok = self.token(
            iter.next(),
            &[TokenKind::ValueRefOrIdent, TokenKind::Number],
        )?;
        if tok.kind == TokenKind::Number {
            comp.number = Some(self.token_value(tok).to_string());
        } else {
            comp.label = Some(self.token_value(tok).to_string());
            if iter.peek().is_some() {
                self.token(iter.next(), TokenKind::LeftParen)?;
                let tok = self.token(iter.next(), TokenKind::Number)?;
                self.token(iter.next(), TokenKind::RightParen)?;
                comp.number = Some(self.token_value(tok).to_string());
            }
        }

        iter.assert_empty()?;

        Ok(comp)
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

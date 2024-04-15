use crate::{
    analysis::AnalysisContext,
    cst::{Asn1Tag, AsnNodeId, CstIter},
    diagnostic::Result,
    token::{Token, TokenKind},
};

use super::{ty_or_value::Type, WithId};

/// A group of ASN.1 assignments and settings.
#[derive(Debug, Clone)]
pub struct ModuleDefinition {
    /// Identifier for the module
    pub identifier: ModuleIdentifier,

    /// Name of the default encoding
    pub encoding_reference: Option<WithId<Token>>,

    /// How automatic tagging should be performed
    pub tag_default: Option<WithId<TagDefault>>,

    /// Is extensibility implied in this module
    pub extensibility: bool,

    /// assigned values within the module
    pub assignments: Vec<Assignment>,
}

#[derive(Debug, Clone)]
pub struct ModuleIdentifier {
    /// Identifier for the module
    pub name: WithId<String>,

    /// The module's object identifier
    pub oid: Option<ModuleOid>,

    /// The module's internationalized resource identifier value
    pub iri: Option<WithId<String>>,
}

/// An object identifier
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModuleOid {
    /// Ordered list of all components in the OID
    pub components: Vec<ModuleOidComponent>,

    /// Node ID for this oid
    pub id: AsnNodeId,
}

/// A single component of an object identifier
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModuleOidComponent {
    /// the non-integer label for the component
    pub label: Option<WithId<String>>,

    /// the integer label for the component.  note that this is not a number as
    /// math should not be done to it, it is an identifier.
    pub number: Option<WithId<String>>,
}

/// How tags are applied to a module
#[derive(Debug, Clone, Copy)]
pub enum TagDefault {
    /// All tagging is automatically calculated
    Automatic,

    /// Tags are by default specified as implicit
    Implicit,

    /// Tags are by default specified as explicit
    Explicit,
}

/// A single assignment from a name to a type or value
#[derive(Debug, Clone)]
pub struct Assignment {
    /// The name being assigned to
    pub name: WithId<String>,

    /// The value or type assigned to the name.
    pub value: WithId<Type>,

    /// A specified type within the assignment
    pub ty: Option<WithId<()>>,

    /// The sort of assignment, how to interpret the values
    pub kind: AssignmentKind,
}

/// What sort of assignment is this
#[derive(Debug, Clone)]
pub enum AssignmentKind {
    TypeAssignment,
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

        let assignments = self.assignments(&mut iter)?;
        self.token(iter.next(), TokenKind::KwEnd)?;
        iter.assert_empty()?;

        Ok(ModuleDefinition {
            identifier: name,
            encoding_reference,
            tag_default,
            extensibility,
            assignments,
        })
    }

    /// Interpret a module identifier
    fn module_identifier(&self, iter: &mut CstIter) -> Result<ModuleIdentifier> {
        let mut iter = self.tree(iter.next(), Asn1Tag::ModuleIdentifier)?;
        let name = self.token(iter.next(), TokenKind::TypeOrModuleRef)?;
        let name = WithId {
            value: self.ident_value(*name),
            id: name.id,
        };

        let oid = if iter.peek().is_some() {
            Some(self.module_oid(&mut iter)?)
        } else {
            None
        };
        let iri = if iter.peek().is_some() {
            let tok = self.token(iter.next(), TokenKind::CString)?;
            Some(WithId {
                value: self.token_value(*tok).to_string(),
                id: tok.id,
            })
        } else {
            None
        };

        iter.assert_empty()?;

        Ok(ModuleIdentifier { name, oid, iri })
    }

    /// Interpret a module identifier's oid
    fn module_oid(&self, iter: &mut CstIter) -> Result<ModuleOid> {
        let mut iter = self.tree(iter.next(), Asn1Tag::DefinitiveOID)?;
        self.token(iter.next(), TokenKind::LeftCurly)?;

        let mut oid = vec![];
        while let Ok(mut comp) = self.tree(iter.peek(), Asn1Tag::DefinitiveOIDComponent) {
            iter.next();
            let comp = self.module_oid_component(&mut comp)?;
            oid.push(comp);
        }

        self.token(iter.next(), TokenKind::RightCurly)?;

        Ok(ModuleOid {
            components: oid,
            id: iter.node,
        })
    }

    /// Interpret the inside of a definitive oid component
    fn module_oid_component(&self, iter: &mut CstIter) -> Result<ModuleOidComponent> {
        let mut comp = ModuleOidComponent {
            label: None,
            number: None,
        };
        let tok = self.token(
            iter.next(),
            &[TokenKind::ValueRefOrIdent, TokenKind::Number],
        )?;
        if tok.kind == TokenKind::Number {
            comp.number = Some(WithId {
                value: self.token_value(*tok).to_string(),
                id: tok.id,
            });
        } else {
            comp.label = Some(WithId {
                value: self.ident_value(*tok),
                id: tok.id,
            });
            if iter.peek().is_some() {
                self.token(iter.next(), TokenKind::LeftParen)?;
                let tok = self.token(iter.next(), TokenKind::Number)?;
                self.token(iter.next(), TokenKind::RightParen)?;
                comp.number = Some(WithId {
                    value: self.token_value(*tok).to_string(),
                    id: tok.id,
                });
            }
        }

        iter.assert_empty()?;

        Ok(comp)
    }

    /// Interpret the encoding reference cst node
    fn encoding_reference(&self, iter: &mut CstIter) -> Result<Option<WithId<Token>>> {
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
    fn tag_default(&self, iter: &mut CstIter) -> Result<Option<WithId<TagDefault>>> {
        let mut iter = self.tree(iter.next(), Asn1Tag::TagDefault)?;
        if iter.peek().is_none() {
            return Ok(None);
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

        let res = match tok.kind {
            TokenKind::KwImplicit => TagDefault::Implicit,
            TokenKind::KwExplicit => TagDefault::Explicit,
            _ => TagDefault::Automatic,
        };

        Ok(Some(WithId {
            value: res,
            id: iter.node,
        }))
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

    /// Gather the list of assignments
    fn assignments(&self, iter: &mut CstIter) -> Result<Vec<Assignment>> {
        let mut res = vec![];

        while let Some(node) = iter.peek() {
            let Ok(mut assign) = self.tree(node, Asn1Tag::Assignment) else {
                break;
            };
            iter.next();

            res.push(self.assignment(&mut assign)?);
        }

        Ok(res)
    }

    /// Parse a single assignment statement
    fn assignment(&self, iter: &mut CstIter) -> Result<Assignment> {
        let name = self.token(
            iter.next(),
            &[TokenKind::TypeOrModuleRef, TokenKind::ValueRefOrIdent],
        )?;

        let (value, kind) = if name.kind == TokenKind::TypeOrModuleRef {
            self.type_assignment(iter)?
        } else {
            todo!()
        };

        Ok(Assignment {
            name: WithId {
                value: self.token_value(*name).to_string(),
                id: name.id,
            },
            value,
            ty: None,
            kind,
        })
    }
}

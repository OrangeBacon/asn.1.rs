//! Type checking and name resolution for ASN.1.
//! The following analysis passes are defined:
//! - Local: analyse each module in isolation to get its imports and exports.
//! - Global: resolve dependencies between modules (imports and exports).
//! - Name: resolve names and references within each module.
//! - Type: resolve types across all modules.
//! - Value: parse and analyse values now that the type of the value is known.
//! Note that modules can depend upon each other and must be checked at the
//! same time so circular and recursive dependency resolution can take place.

mod error;
mod module;

use crate::{
    compiler::SourceId,
    cst::{Asn1, Asn1Tag, AsnNodeId},
    token::{Token, TokenKind},
};

pub use self::error::{AnalysisError, Result};

/// State for analysing syntax trees
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Analysis<'a> {
    /// Text value of the source, used to resolve references within the CST.
    source: &'a str,

    /// Parsed CST for the source file.
    cst: &'a mut Asn1,

    /// ID of the source file.
    id: SourceId,
}

impl<'a> Analysis<'a> {
    /// Create an analysis context for the given source file
    pub fn new(cst: &'a mut Asn1, source: &'a str, id: SourceId) -> Self {
        Self { source, cst, id }
    }

    /// Run module-local analysis to gather imports / exports and other requirements
    /// that do not need full name and type resolution
    pub fn local(&mut self) -> Result<(), AnalysisError> {
        let (_, root) = self.get_tree(self.cst.root, &[Asn1Tag::Root])?;

        for module in root {
            if self.is_comment(module) {
                continue;
            }

            self.local_module(module)?;
        }

        Ok(())
    }

    /// Get a list of nodes that are contained within a given node and return the
    /// tag of that node.  If the tag does not match one of the provided kinds,
    /// returns an error.
    fn get_tree(
        &self,
        node: AsnNodeId,
        kind: &[Asn1Tag],
    ) -> Result<(Asn1Tag, impl Iterator<Item = AsnNodeId>)> {
        let tag = self
            .cst
            .tree_tag(node)
            .ok_or_else(|| AnalysisError::NotTree {
                node,
                id: self.id,
                expected: kind.to_vec(),
            })?;

        if !kind.is_empty() && !kind.contains(&tag) {
            return Err(AnalysisError::WrongTree {
                node,
                id: self.id,
                expected: kind.to_vec(),
                got: tag,
            });
        }

        let iter = self
            .cst
            .iter_tree(node)
            .ok_or_else(|| AnalysisError::NotTree {
                node,
                id: self.id,
                expected: kind.to_vec(),
            })?;

        Ok((tag, iter))
    }

    /// Is the given node a comment token
    fn is_comment(&self, node: AsnNodeId) -> bool {
        matches!(
            self.cst.token(node),
            Some(Token {
                kind: TokenKind::SingleComment | TokenKind::MultiComment,
                ..
            })
        )
    }
}

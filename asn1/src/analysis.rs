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

use std::fmt::Write;

use crate::{
    compiler::SourceId,
    cst::{Asn1, Asn1Tag, AsnNodeId, CstIter},
    token::{Token, TokenKind},
    util::CowVec,
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
    pub(crate) id: SourceId,
}

impl<'a> Analysis<'a> {
    /// Create an analysis context for the given source file
    pub fn new(cst: &'a mut Asn1, source: &'a str, id: SourceId) -> Self {
        Self { source, cst, id }
    }

    /// Run module-local analysis to gather imports / exports and other requirements
    /// that do not need full name and type resolution
    pub fn local(&mut self) -> Result {
        let root = self.tree(self.cst.root, &[Asn1Tag::Root])?;

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
    /// returns None.
    pub fn tree(
        &self,
        node: impl Into<Option<AsnNodeId>>,
        asn1_tag: impl Into<CowVec<Asn1Tag>>,
    ) -> Result<CstIter> {
        let kind = asn1_tag.into();

        let Some(node) = node.into() else {
            return Err(AnalysisError::NoTreeNode {
                id: self.id,
                expected: kind,
            });
        };

        let Some(tag) = self.cst.tree_tag(node) else {
            return Err(AnalysisError::NotTree {
                node,
                id: self.id,
                expected: kind,
            });
        };

        if !kind.is_empty() && !kind.contains(&tag) {
            return Err(AnalysisError::WrongTree {
                node,
                id: self.id,
                expected: kind,
                got: tag,
            });
        }

        let iter = self.cst.iter_tree(node).ok_or(AnalysisError::NotTree {
            node,
            id: self.id,
            expected: kind,
        })?;

        Ok(iter)
    }

    pub fn token(
        &self,
        node: impl Into<Option<AsnNodeId>>,
        token_kind: impl Into<CowVec<TokenKind>>,
    ) -> Result<Token> {
        let kind = token_kind.into();

        let Some(node) = node.into() else {
            return Err(AnalysisError::NoTokenNode {
                id: self.id,
                expected: kind,
            });
        };

        let Some(tok) = self.cst.token(node) else {
            return Err(AnalysisError::NotToken {
                node,
                id: self.id,
                expected: kind,
            });
        };

        if !kind.is_empty() && !kind.contains(&tok.kind) {
            return Err(AnalysisError::WrongToken {
                node,
                id: self.id,
                expected: kind,
                got: tok.kind,
            });
        }

        Ok(tok)
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

    /// Convert a token into a user-readable string (debugging method)
    pub fn token_string(&self, tok: Token) -> String {
        debug_assert_eq!(tok.id, self.id);

        let mut s = String::new();

        write!(
            s,
            "{:?}@{}..{}: {}",
            tok.kind,
            tok.offset,
            tok.offset + tok.length,
            &self.source[tok.offset..tok.offset + tok.length]
        )
        .unwrap();

        s
    }
}

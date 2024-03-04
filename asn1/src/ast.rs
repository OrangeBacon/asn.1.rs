mod error;
mod module;

use crate::{
    analysis::AnalysisContext,
    cst::{Asn1Tag, AsnNodeId, CstIter},
    token::{Token, TokenKind},
    util::CowVec,
};

pub use self::error::{AstError, Result};

impl AnalysisContext<'_> {
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
            return Err(AstError::NoTreeNode { expected: kind });
        };

        let source = self.source(node.source());
        let cst = &source.tree;

        let Some(tag) = cst.tree_tag(node) else {
            return Err(AstError::NotTree {
                node,
                id: source.id,
                expected: kind,
            });
        };

        if !kind.is_empty() && !kind.contains(&tag) {
            return Err(AstError::WrongTree {
                node,
                id: source.id,
                expected: kind,
                got: tag,
            });
        }

        let iter = cst.iter_tree(node).ok_or(AstError::NotTree {
            node,
            id: source.id,
            expected: kind,
        })?;

        Ok(iter)
    }

    /// Get a token of one of the provided kinds from the given tree node id.
    pub fn token(
        &self,
        node: impl Into<Option<AsnNodeId>>,
        token_kind: impl Into<CowVec<TokenKind>>,
    ) -> Result<Token> {
        let kind = token_kind.into();

        let Some(node) = node.into() else {
            return Err(AstError::NoTokenNode { expected: kind });
        };

        let source = self.source(node.source());
        let cst = &source.tree;

        let Some(tok) = cst.token(node) else {
            return Err(AstError::NotToken {
                node,
                id: source.id,
                expected: kind,
            });
        };

        if !kind.is_empty() && !kind.contains(&tok.kind) {
            return Err(AstError::WrongToken {
                node,
                id: source.id,
                expected: kind,
                got: tok.kind,
            });
        }

        Ok(tok)
    }

    /// Is the given node a comment token
    pub fn is_comment(&self, node: AsnNodeId) -> bool {
        matches!(
            self.source(node.source()).tree.token(node),
            Some(Token {
                kind: TokenKind::SingleComment | TokenKind::MultiComment,
                ..
            })
        )
    }

    /// Get the string value of a token
    pub fn token_value(&self, tok: Token) -> &str {
        &self.source(tok.id).source[tok.offset..tok.offset + tok.length]
    }
}

impl CstIter<'_> {
    /// Are there any more nodes in this iterator
    pub fn assert_empty(&mut self) -> Result {
        if let Some(id) = self.peek() {
            Err(AstError::FoundNode {
                id: self.id,
                got: id,
            })
        } else {
            Ok(())
        }
    }
}

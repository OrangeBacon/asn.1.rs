//! Type checking and name resolution for ASN.1

mod error;

use std::ops::Range;

use crate::{
    compiler::SourceId,
    cst::{Asn1, Asn1Tag, TreeContent},
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
        }

        Ok(())
    }

    /// Get a list of nodes that are contained within given node
    fn get_tree(&mut self, node: usize, kind: &[Asn1Tag]) -> Result<(Asn1Tag, Range<usize>)> {
        match self.cst.data[node] {
            TreeContent::Tree { tag, start, count } => {
                if kind.is_empty() || kind.contains(&tag) {
                    Ok((tag, start..start + count))
                } else {
                    Err(AnalysisError::WrongTree {
                        node,
                        id: self.id,
                        expected: kind.to_vec(),
                        got: tag,
                    })
                }
            }
            TreeContent::Token(_) => Err(AnalysisError::NotTree { node, id: self.id, expected: kind.to_vec() }),
        }
    }

    /// Is the given node a comment token
    fn is_comment(&mut self, node: usize) -> bool {
        matches!(
            self.cst.data[node],
            TreeContent::Token(Token {
                kind: TokenKind::SingleComment | TokenKind::MultiComment,
                ..
            })
        )
    }
}

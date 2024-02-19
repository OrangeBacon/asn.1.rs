use crate::{
    compiler::SourceId,
    cst::{Asn1Tag, AsnNodeId},
    token::TokenKind,
    util::CowVec,
};

/// Any error that can be produced by an analysis pass
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AnalysisError {
    /// Expected a tree node but found a token node
    NotTree {
        node: AsnNodeId,
        id: SourceId,
        expected: CowVec<Asn1Tag>,
    },

    /// Expected a token node but found a tree node
    NotToken {
        node: AsnNodeId,
        id: SourceId,
        expected: CowVec<TokenKind>,
    },

    /// Expected a tree node but found nothing (internal error)
    NoTreeNode {
        id: SourceId,
        expected: CowVec<Asn1Tag>,
    },

    /// Expected a token node but found nothing
    NoTokenNode {
        id: SourceId,
        expected: CowVec<TokenKind>,
    },

    /// Expected a tree of the given kind, but got something different
    WrongTree {
        node: AsnNodeId,
        id: SourceId,
        expected: CowVec<Asn1Tag>,
        got: Asn1Tag,
    },

    /// Expected a tree of the given kind, but got something different
    WrongToken {
        node: AsnNodeId,
        id: SourceId,
        expected: CowVec<TokenKind>,
        got: TokenKind,
    },
}

pub type Result<T = (), E = AnalysisError> = std::result::Result<T, E>;

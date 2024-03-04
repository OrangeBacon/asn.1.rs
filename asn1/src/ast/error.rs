use crate::{
    analysis::IriParseError, compiler::SourceId, cst::{Asn1Tag, AsnNodeId}, token::TokenKind, util::CowVec
};

/// Any error that can be produced by ast construction.  If thrown to the user,
/// this would be a bug in our code, not the user's code.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AstError {
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

    /// Expected a tree node but found nothing
    NoTreeNode { expected: CowVec<Asn1Tag> },

    /// Expected a token node but found nothing
    NoTokenNode { expected: CowVec<TokenKind> },

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

    /// Expected an iterator to be empty but it was not
    FoundNode { id: SourceId, got: AsnNodeId },

    /// An error occurred while parsing an IRI string
    IriParseError { err: IriParseError, node: AsnNodeId },
}

pub type Result<T = (), E = AstError> = std::result::Result<T, E>;

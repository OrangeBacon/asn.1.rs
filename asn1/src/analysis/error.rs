use crate::{compiler::SourceId, cst::Asn1Tag};

/// Any error that can be produced by an analysis pass
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AnalysisError {
    /// Expected a tree node but found a token node
    NotTree {
        node: usize,
        id: SourceId,
        expected: Vec<Asn1Tag>,
    },

    /// Expected a tree of the given kind, but got something different
    WrongTree {
        node: usize,
        id: SourceId,
        expected: Vec<Asn1Tag>,
        got: Asn1Tag,
    },
}

pub type Result<T = (), E = AnalysisError> = std::result::Result<T, E>;

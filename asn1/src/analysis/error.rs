use crate::{ast::AstError, cst::AsnNodeId};

/// Any error that can be produced by an analysis pass
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AnalysisError {
    /// Internal Compiler Error while constructing an AST node
    AstError { source: AstError },

    /// A duplicate module name was found
    DuplicateModule { first: AsnNodeId, second: AsnNodeId },
}

pub type Result<T = (), E = AnalysisError> = std::result::Result<T, E>;

impl From<AstError> for AnalysisError {
    fn from(value: AstError) -> Self {
        AnalysisError::AstError { source: value }
    }
}

/// Any warning that can be produced by an analysis pass
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AnalysisWarning {
    /// A duplicate module name was found, however the definitive IDs were different.
    DuplicateModule { first: AsnNodeId, second: AsnNodeId },
}

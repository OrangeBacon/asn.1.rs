use crate::ast::AstError;

/// Any error that can be produced by an analysis pass
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AnalysisError {
    /// Internal Compiler Error while constructing an AST node
    AstError { source: AstError },
}

pub type Result<T = (), E = AnalysisError> = std::result::Result<T, E>;

impl From<AstError> for AnalysisError {
    fn from(value: AstError) -> Self {
        AnalysisError::AstError { source: value }
    }
}

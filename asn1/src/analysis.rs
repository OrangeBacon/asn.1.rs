//! Type checking and name resolution for ASN.1

mod error;

use crate::cst::Asn1;

pub use self::error::AnalysisError;

/// State for analysing syntax trees
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Analysis<'a> {
    source: &'a str,
    cst: &'a mut Asn1,
}

impl<'a> Analysis<'a> {
    /// Create an analysis context for the given source file
    pub fn new(cst: &'a mut Asn1, source: &'a str) -> Self {
        Self { source, cst }
    }

    /// Run module-local analysis to gather imports / exports and other requirements
    /// that do not need full name and type resolution
    pub fn local(&mut self) -> Result<(), AnalysisError> {
        Ok(())
    }
}

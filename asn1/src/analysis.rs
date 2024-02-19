//! Type checking and name resolution for ASN.1.
//! The following analysis passes are defined:
//! - Local: analyse each module in isolation to get its imports and exports.
//! - Global: resolve dependencies between modules (imports and exports).
//! - Name: resolve names and references within each module.
//! - Type: resolve types across all modules.
//! - Value: parse and analyse values now that the type of the value is known.
//! Note that modules can depend upon each other and must be checked at the
//! same time so circular and recursive dependency resolution can take place.
//!
//! The analysis passes are all based on the fact that the provided CST from the
//! parser is valid as far as the parser can tell.  Any parse errors that put
//! tokens in weird locations within the cst due to error recovery, etc (not
//! currently implemented) should ensure that either analysis does not run or
//! the recovered ast is valid.  Otherwise an internal compiler error will be
//! thrown.  If the error is that a structure that should not be present is,
//! however it could not be detected in parsing, then it likely will be thrown
//! as a type error for the user to fix and analysis to continue.

mod error;
mod module;

use crate::{
    compiler::SourceId,
    cst::{Asn1, Asn1Tag},
};

pub use self::error::{AnalysisError, Result};

/// State for analysing syntax trees
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Analysis<'a> {
    /// Text value of the source, used to resolve references within the CST.
    pub(crate) source: &'a str,

    /// Parsed CST for the source file.
    pub(crate) cst: &'a mut Asn1,

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
}

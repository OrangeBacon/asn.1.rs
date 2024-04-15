use crate::diagnostic::Result;

use super::context::AnalysisContext;

impl AnalysisContext<'_> {
    /// Run global analysis to resolve module names/imports/exports
    pub(super) fn global(&mut self) -> Result {
        Ok(())
    }
}

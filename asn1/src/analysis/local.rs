use crate::{compiler::SourceId, cst::Asn1Tag};

use super::{context::AnalysisContext, error::Result};

impl AnalysisContext<'_> {
    /// Run module-local analysis to gather imports / exports and other requirements
    /// that do not need full name and type resolution
    pub(super) fn local(&mut self, file: SourceId) -> Result {
        let cst = self.source(file);
        let mut root = self.tree(cst.tree.root, &[Asn1Tag::Root])?;

        let mut modules = vec![];
        while let Some(module) = root.next() {
            if self.is_comment(module) {
                continue;
            }

            modules.push(module);
        }

        self.modules.extend(modules);

        Ok(())
    }
}

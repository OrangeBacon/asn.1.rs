use super::{context::AnalysisContext, error::Result};

impl AnalysisContext<'_> {
    /// Run global analysis to resolve module names/imports/exports
    pub(super) fn global(&mut self) -> Result {
        for module in &self.modules {
            let ast = self.module_ast(module.node)?;
            println!("{ast:?}");
        }

        Ok(())
    }
}

use super::{context::AnalysisContext, error::Result, AnalysisError};

impl AnalysisContext<'_> {
    /// Run global analysis to resolve module names/imports/exports
    pub(super) fn global(&mut self) -> Result {
        let modules = 0..self.modules.len();
        for module in modules {
            let ast = self.module_ast(self.modules[module].node)?;

            let identifier = ast.identifier;
            let module_id = identifier.name.value.to_string();

            if let Some(id) = identifier.iri {
                return Err(AnalysisError::OidIriUnsupported { id: id.id });
            }
            if let Some(id) = identifier.oid {
                return Err(AnalysisError::OidIriUnsupported { id: id.id });
            }

            let module = &mut self.modules[module];
            module.name = module_id;
        }

        Ok(())
    }
}

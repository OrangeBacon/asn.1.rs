use crate::diagnostic::Result;

use super::context::AnalysisContext;

impl AnalysisContext<'_> {
    /// Run global analysis to resolve module names/imports/exports
    pub(super) fn global(&mut self) -> Result {
        let modules = 0..self.modules.len();
        for module in modules {
            let ast = self.module_ast(self.modules[module].node)?;

            let identifier = ast.identifier;
            let module_id = identifier.name.value.to_string();

            if let Some(iri) = identifier.iri {
                todo!("iri: {iri:?}")
            }
            if let Some(oid) = identifier.oid {
                todo!("oid: {oid:?}")
            }

            let module = &mut self.modules[module];
            module.name = module_id;
        }

        Ok(())
    }
}

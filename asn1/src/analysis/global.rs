use crate::{diagnostic::Result, Diagnostic};

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
                let id = iri.id;

                self.diagnostics.push(
                    Diagnostic::error("Asn1::Analysis::Iri")
                        .name("Module IRI Descriptors are not supported")
                        .label(self.label(id).message("IRI included here")),
                );
                continue;
            }

            if let Some(oid) = identifier.oid {
                let id = oid.id;

                self.diagnostics.push(
                    Diagnostic::error("Asn1::Analysis::OID")
                        .name("Module OID Descriptors are not supported")
                        .label(self.label(id).message("OID included here")),
                );
                continue;
            }

            let module = &mut self.modules[module];
            module.name = module_id;
        }

        Ok(())
    }
}

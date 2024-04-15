use crate::{
    compiler::SourceId,
    cst::{Asn1Tag, AsnNodeId},
    diagnostic::Result,
    Diagnostic,
};

use super::{context::AnalysisContext, environment::Environment};

impl AnalysisContext<'_> {
    /// Run module-local analysis to gather imports / exports and other requirements
    /// that do not need full name and type resolution
    pub(super) fn local(&mut self, file: SourceId) -> Result {
        let cst = self.source(file);
        let mut root = self.tree(cst.tree.root, &[Asn1Tag::Root])?;

        let mut ids = vec![];
        while let Some(module) = root.next() {
            ids.push(module);
        }

        let mut modules = Vec::with_capacity(ids.len());
        for module in ids {
            modules.push(self.local_module(module)?);
        }

        self.modules.extend(modules);

        Ok(())
    }

    fn local_module(&mut self, module: AsnNodeId) -> Result<Environment> {
        let ast = self.module_ast(module)?;

        let identifier = ast.identifier;
        let module_id = identifier.name.value.to_string();

        if let Some(iri) = identifier.iri {
            let id = iri.id;

            self.diagnostics.push(
                Diagnostic::error("Asn1::Analysis::Iri")
                    .name("Module IRI Descriptors are not supported")
                    .label(self.label(id).message("IRI included here")),
            );
        }

        if let Some(oid) = identifier.oid {
            let id = oid.id;

            self.diagnostics.push(
                Diagnostic::error("Asn1::Analysis::OID")
                    .name("Module OID Descriptors are not supported")
                    .label(self.label(id).message("OID included here")),
            );
        }

        let mut module = Environment::new(module);
        module.name = module_id;

        for assign in ast.assignments {
            module.variables.insert(assign.name.value, ());
        }

        Ok(module)
    }
}

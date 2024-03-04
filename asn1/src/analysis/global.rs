use super::{context::AnalysisContext, error::Result, AnalysisError, Iri};

impl AnalysisContext<'_> {
    /// Run global analysis to resolve module names/imports/exports
    pub(super) fn global(&mut self) -> Result {
        let modules = 0..self.modules.len();
        for module in modules {
            let ast = self.module_ast(self.modules[module].node)?;
            println!("{ast:?}");

            let name = ast.name;
            let module_id = name.name.value.to_string();
            //let oid = name.oid;
            let iri = if let Some(node) = name.iri {
                let iri = Iri::from_str(node.value);
                match iri {
                    Ok(iri) => Some(iri),
                    Err(err) => {
                        self.errors
                            .push(AnalysisError::IriParseError { err, node: node.id });
                        None
                    }
                }
            } else {
                None
            };

            let module = &mut self.modules[module];
            module.name = module_id;
            //module.oid = oid;
            module.iri = iri;
        }

        Ok(())
    }
}

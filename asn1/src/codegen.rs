use std::fmt::Write;

use convert_case::{Case, Casing};

use crate::analysis::{AnalysisContext, Environment};

/// Generate rust source code from a successful analysis context
#[derive(Debug, Clone)]
struct RustCodegen<'a> {
    analysis: &'a AnalysisContext<'a>,
    result: String,
}

#[derive(Debug, Clone, Copy)]
pub enum CodegenError {
    /// Cannot run codegen if any errors are present in the analysis context
    AnalysisErrors,

    /// Error during write to string? should never occur
    FmtError(std::fmt::Error),
}

type Result<T = (), E = CodegenError> = std::result::Result<T, E>;

impl AnalysisContext<'_> {
    /// Run the code generator to produce a rust source code listing to represent the input files.
    pub fn rust_codegen(&self) -> Result<String> {
        if !self.diagnostics.is_empty() {
            return Err(CodegenError::AnalysisErrors);
        }

        RustCodegen {
            analysis: self,
            result: String::new(),
        }
        .run()
    }
}

impl RustCodegen<'_> {
    fn run(mut self) -> Result<String> {
        for module in &self.analysis.modules {
            self.module(module)?;
        }

        Ok(self.result)
    }

    fn module(&mut self, module: &Environment) -> Result {
        writeln!(self.result, "mod {} {{", module.name.to_case(Case::Snake))?;

        for var in module.variables.keys() {
            writeln!(
                self.result,
                "\tconst {}: () = ();",
                var.to_case(Case::ScreamingSnake)
            )?;
        }

        writeln!(self.result, "}}")?;

        Ok(())
    }
}

impl From<std::fmt::Error> for CodegenError {
    fn from(value: std::fmt::Error) -> Self {
        Self::FmtError(value)
    }
}

use std::ops::{Deref, DerefMut};

use crate::{compiler::SourceId, AsnCompiler};

use super::{local::LocalAnalysis, AnalysisError};

/// Data used and produced by static analysis of source files
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AnalysisContext<'a> {
    compiler: &'a mut AsnCompiler,

    /// All errors that occurred while running the analysis.
    pub errors: Vec<AnalysisError>,
}

impl<'a> AnalysisContext<'a> {
    /// Create a new, blank, analysis context
    pub(crate) fn new(compiler: &'a mut AsnCompiler) -> Self {
        let mut this = Self {
            compiler,
            errors: vec![],
        };

        let sources: Vec<_> = this.compiler.all_sources().collect();

        for file in sources {
            this.add_source(file);
        }

        this
    }

    /// Add a new source file to the context and run local analysis on it.
    fn add_source(&mut self, file: SourceId) {
        if let Err(e) = LocalAnalysis::new(self, file).local() {
            self.errors.push(e);
        }
    }
}

impl Deref for AnalysisContext<'_> {
    type Target = AsnCompiler;

    fn deref(&self) -> &Self::Target {
        self.compiler
    }
}

impl DerefMut for AnalysisContext<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.compiler
    }
}

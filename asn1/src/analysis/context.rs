use std::ops::{Deref, DerefMut};

use crate::{compiler::SourceId, cst::AsnNodeId, AsnCompiler};

use super::{error::AnalysisWarning, AnalysisError};

/// Data used and produced by static analysis of source files
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AnalysisContext<'a> {
    /// context to get source files/trees from
    compiler: &'a mut AsnCompiler,

    /// All errors that occurred while running the analysis.
    pub errors: Vec<AnalysisError>,

    /// All warnings that occurred while running the analysis.
    pub warnings: Vec<AnalysisWarning>,

    /// List of all modules from all source files
    pub(crate) modules: Vec<AsnNodeId>,
}

impl<'a> AnalysisContext<'a> {
    /// Create a new, blank, analysis context
    pub(crate) fn new(compiler: &'a mut AsnCompiler) -> Self {
        let mut this = Self {
            compiler,
            errors: vec![],
            warnings: vec![],
            modules: vec![],
        };

        let sources: Vec<_> = this.compiler.all_sources().collect();

        for file in sources {
            this.add_source(file);
        }

        this.run();

        this
    }

    /// Add a new source file to the context and run local analysis on it.
    fn add_source(&mut self, file: SourceId) {
        if let Err(e) = self.local(file) {
            self.errors.push(e);
        }
    }

    /// Run all analysis passes
    fn run(&mut self) {}
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

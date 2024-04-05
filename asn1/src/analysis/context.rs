use std::ops::{Deref, DerefMut};

use crate::{compiler::SourceId, cst::AsnNodeId, diagnostic::Label, AsnCompiler, Diagnostic};

use super::environment::Environment;

/// Data used and produced by static analysis of source files
#[derive(Debug)]
pub struct AnalysisContext<'a> {
    /// context to get source files/trees from
    compiler: &'a mut AsnCompiler,

    /// All diagnostics that occurred while running the analysis.
    pub diagnostics: Vec<Diagnostic>,

    /// List of all modules from all source files
    pub(crate) modules: Vec<Environment>,
}

impl<'a> AnalysisContext<'a> {
    /// Create a new, blank, analysis context
    pub(crate) fn new(compiler: &'a mut AsnCompiler) -> Self {
        let errors = std::mem::take(&mut compiler.diagnostics);

        let mut this = Self {
            compiler,
            diagnostics: errors,
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
            self.diagnostics.push(e);
        }
    }

    /// Run all analysis passes
    fn run(&mut self) {
        if let Err(e) = self.global() {
            self.diagnostics.push(e);
        }
    }

    /// Construct a diagnostic label that references a given tree node
    pub(crate) fn label(&self, node: AsnNodeId) -> Label {
        self.compiler.source(node.source()).tree.label(node)
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

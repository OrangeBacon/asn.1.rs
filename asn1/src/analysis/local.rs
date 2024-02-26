use crate::{
    compiler::SourceId,
    cst::{Asn1Tag, AsnNodeId},
};

use super::{context::AnalysisContext, error::Result};

/// State for analysing syntax trees
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct LocalAnalysis<'a, 'b> {
    /// Text value of the source, used to resolve references within the CST.
    ctx: &'a mut AnalysisContext<'b>,

    /// ID of the source file.
    id: SourceId,
}

impl<'a, 'b> LocalAnalysis<'a, 'b> {
    /// Create an analysis context for the given source file
    pub fn new(ctx: &'a mut AnalysisContext<'b>, id: SourceId) -> Self {
        Self { ctx, id }
    }

    /// Run module-local analysis to gather imports / exports and other requirements
    /// that do not need full name and type resolution
    pub fn local(&mut self) -> Result {
        let cst = self.ctx.source(self.id);
        let mut root = self.ctx.tree(cst.tree.root, &[Asn1Tag::Root])?;

        while let Some(module) = root.next() {
            if self.ctx.is_comment(module) {
                continue;
            }

            self.module(module)?;
        }

        Ok(())
    }

    fn module(&self, node: AsnNodeId) -> Result {
        let module = self.ctx.module_ast(node)?;

        println!("{:?}", module);

        Ok(())
    }
}

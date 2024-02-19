use crate::cst::AsnNodeId;

use super::{Analysis, Result};

impl Analysis<'_> {
    pub(super) fn local_module(&mut self, node: AsnNodeId) -> Result {
        let module = self.module_ast(node)?;

        println!("{}", self.token_string(module.name));

        Ok(())
    }
}

use crate::cst::{Asn1Tag, AsnNodeId};

use super::{Analysis, Result};

/// A group of ASN.1 assignments and settings.
#[derive(Debug, Clone, Copy, Default)]
struct Module<'a> {
    /// Identifier for the module
    name: &'a str,

    encoding_reference: Option<&'a str>,

    tag: ModuleTag,
}

/// How or whether tags should be generated
#[derive(Debug, Clone, Copy, Default)]
enum ModuleTag {
    #[default]
    Explicit,
    Implicit,
    Automatic,
}

impl<'a> Analysis<'a> {
    pub(super) fn local_module(&mut self, node: AsnNodeId) -> Result {
        self.module(node)?.local()?;

        Ok(())
    }

    fn module(&mut self, node: AsnNodeId) -> Result<Module> {
        let (_, iter) = self.get_tree(node, &[Asn1Tag::ModuleDefinition])?;
        let iter = iter.filter(|e| !self.is_comment(*e));

        // let mut module = Module::default();
        // for node in iter {
        //     let (tag, iter) =
        //         self.get_tree(node, &[Asn1Tag::ModuleIdentifier, Asn1Tag::ModuleDefaults])?;
        // }

        //Ok(module)
        todo!()
    }
}

impl Module<'_> {
    fn local(&self) -> Result {
        Ok(())
    }
}

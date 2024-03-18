use crate::cst::AsnNodeId;

// use super::{Iri, Oid};

/// Local variable resolution environment
#[derive(Debug, PartialEq, Eq)]
pub struct Environment {
    /// ID of the tree node for the whole module
    pub node: AsnNodeId,

    /// The initially defined module name (hopefully but not necessarily unique)
    pub name: String,

    // /// The object identifier for the module (if present)
    // pub oid: Option<Oid>,

    // /// The iri of the module (if present)
    // pub iri: Option<Iri>,
}

impl Environment {
    /// Create a new empty environment for a given module
    pub fn new(node: AsnNodeId) -> Environment {
        Environment {
            node,
            name: String::new(),
            // oid: None,
            // iri: None,
        }
    }
}

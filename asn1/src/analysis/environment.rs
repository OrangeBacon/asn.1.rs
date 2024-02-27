use crate::cst::AsnNodeId;

/// Local variable resolution environment
#[derive(Debug, PartialEq, Eq)]
pub struct Environment {
    /// ID of the tree node for the whole module
    pub node: AsnNodeId,
}

impl Environment {
    /// Create a new empty environment for a given module
    pub fn new(node: AsnNodeId) -> Environment {
        Environment { node }
    }
}

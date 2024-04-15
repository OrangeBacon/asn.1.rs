use std::collections::HashMap;

use crate::{
    ast::{Type, WithId},
    cst::AsnNodeId,
};

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
    /// All variables defined within the module
    pub variables: HashMap<String, Variable>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Variable {
    /// Node ID of the location the variable was defined
    pub id: AsnNodeId,

    /// The value assigned to the variable
    pub value: WithId<Type>,

    /// The Type associated with the variable, if specified
    pub ty: Option<()>,
}

impl Environment {
    /// Create a new empty environment for a given module
    pub fn new(node: AsnNodeId) -> Environment {
        Environment {
            node,
            name: String::new(),
            // oid: None,
            // iri: None,
            variables: HashMap::new(),
        }
    }
}

//! Representation of a parsed ASN.1 description file (NOT an encoded message)

// --------------------
// 13 Module Definition
// --------------------
// WIP

use crate::token::TokenBuffer;

/// A whole ASN.1 file including all modules
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Asn1 {
    /// All included modules in the file
    pub modules: Vec<ModuleDefinition>,
}

/// Definition of a single module
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModuleDefinition {
    /// All assignments within the module body
    pub assignments: Vec<Assignment>,
}

/// A single assignment within a module body
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Assignment {
    Type(TypeAssignment),
    Value(ValueAssignment),
}

// --------------------------
// 16 Assigning Types and Values
// --------------------------
// WIP

/// 16.1 Assignment of a type to a name
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeAssignment {
    /// The name to be defined
    pub type_reference: TokenBuffer,

    /// The type definition being assigned to the name
    pub ty: Type,
}

/// 16.2 Assignment of a value to a name
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ValueAssignment {
    /// The name to be defined
    pub type_reference: TokenBuffer,

    /// The type of the value
    pub ty: Type,

    /// The value being assigned to the name
    pub value: Value,
}

// ---------------------------------
// 17 Definition of Types and Values
// ---------------------------------
// WIP

/// A type specifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
    Boolean,
}

/// A value of a given type
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value {
    Boolean(bool),
}

// --------------------------------
// 18 Notation for the Boolean Type
// --------------------------------

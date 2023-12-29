//! Representation of a parsed ASN.1 description file (NOT an encoded message)

use std::fmt::Display;

use crate::token::Token;

/// A whole ASN.1 file including all modules
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Asn1<'a> {
    pub root: usize,
    pub data: Vec<TreeContent<'a>>,
}

/// Content of a tree
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TreeContent<'a> {
    Tree {
        tag: Asn1Tag,
        start: usize,
        count: usize,
    },
    Token(Token<'a>),
}

/// The possible kinds of tree node
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Asn1Tag {
    // parser
    Root,

    // module
    ModuleDefinition,
    ModuleIdentifier,
    ModuleDefaults,
    DefinitiveOID,
    EncodingReferenceDefault,
    TagDefault,
    ExtensionDefault,
    Assignment,
    TypeAssignment,
    ValueAssignment,
    Exports,

    // type
    Type,
    IntegerType,
    EnumeratedType,
    EnumItemList,
    EnumItem,
    ExceptionSpec,
    NamedNumber,

    // value
    Value,
    DefinedValue,
    IntegerValue,
    IriValue,
    ExternalValueReference,

    // xml value
    XMLTypedValue,
    XMLTag,
    XMLValue,
    XMLBoolean,
    XMLInteger,
    XMLIri,
}

impl Display for Asn1<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt = Asn1Formatter {
            depth: 0,
            tree: self,
            node: self.data[self.root],
            prefix: String::new(),
            child_prefix: String::new(),
        };

        write!(f, "{fmt}")
    }
}

struct Asn1Formatter<'a, 'b> {
    depth: usize,
    tree: &'a Asn1<'b>,
    node: TreeContent<'b>,
    prefix: String,
    child_prefix: String,
}

impl Display for Asn1Formatter<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.prefix)?;

        match self.node {
            TreeContent::Tree { tag, start, count } => {
                write!(f, "{:?}:", tag)?;
                let Some((last, head)) = self.tree.data[start..start + count].split_last() else {
                    writeln!(f, " (empty)")?;
                    return Ok(());
                };

                writeln!(f)?;

                for &node in head {
                    let fmt = Asn1Formatter {
                        depth: self.depth + 1,
                        tree: self.tree,
                        node,
                        prefix: self.child_prefix.clone() + "|-- ",
                        child_prefix: self.child_prefix.clone() + "|   ",
                    };

                    write!(f, "{fmt}")?;
                }

                let fmt = Asn1Formatter {
                    depth: self.depth + 1,
                    tree: self.tree,
                    node: *last,
                    prefix: self.child_prefix.clone() + "`-- ",
                    child_prefix: self.child_prefix.clone() + "    ",
                };

                write!(f, "{fmt}")?;
            }
            TreeContent::Token(t) => writeln!(f, "{:?}: {:?}", t.kind, t.value)?,
        }

        Ok(())
    }
}

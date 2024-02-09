//! Representation of a parsed ASN.1 description file (NOT an encoded message)

use std::fmt::Display;

use crate::token::Token;

/// A whole ASN.1 file including all modules
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Asn1 {
    pub root: usize,
    pub data: Vec<TreeContent>,
}

/// Content of a tree
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TreeContent {
    Tree {
        tag: Asn1Tag,
        start: usize,
        count: usize,
    },
    Token(Token),
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
    DefinitiveOIDComponent,
    EncodingReferenceDefault,
    TagDefault,
    ExtensionDefault,
    Assignment,
    TypeAssignment,
    ValueAssignment,
    Exports,
    Imports,
    EncodingControl,
    EncodingControlSection,

    // type or value
    TypeOrValue,
    Defined,
    FieldNames,

    // type
    IntegerType,
    EnumeratedType,
    EnumItemList,
    EnumItem,
    ExceptionSpec,
    NamedNumber,
    ObjectIDType,
    SelectionType,
    BitStringType,
    OctetStringType,
    ObjectFields,
    InstanceOfType,
    EmbeddedPDVType,
    PrefixType,

    // composite type
    SequenceType,
    SetType,
    ExtensionAndException,
    ComponentTypeList,
    ComponentType,
    ExtensionAdditions,
    ExtensionAddition,
    ExtensionAdditionGroup,
    VersionNumber,
    ChoiceType,
    TypeList,
    ChoiceExtension,
    ChoiceExtensionList,
    ChoiceExtensionItem,

    // object
    ObjectClass,
    FieldSpecList,
    FieldSpec,
    TypeFieldSpec,
    ValueFieldSpec,
    OptionalitySpec,
    SyntaxSpec,
    SyntaxSpecList,
    OptionalSyntaxSpec,

    // value
    NumberValue,
    BracedValue,
    ChoiceValue,
    ContainingValue,
    OpenTypeFieldValue,

    // xml value
    XMLValue,
    XMLTag,
    XMLData,

    // reference
    SymbolList,
    Symbol,
    Reference,
    SymbolsFromModuleList,

    // parameterized
    ActualParameterList,
}

/// Formatter for the CST of an asn1 file
pub(crate) struct Asn1Formatter<'a> {
    pub tree: &'a Asn1,
    pub source: &'a str,
}

impl Display for Asn1Formatter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt = Asn1FormatterInternal {
            depth: 0,
            tree: self.tree,
            node: self.tree.data[self.tree.root],
            prefix: String::new(),
            child_prefix: String::new(),
            source: self.source,
        };

        write!(f, "{fmt}")
    }
}

struct Asn1FormatterInternal<'a> {
    depth: usize,
    tree: &'a Asn1,
    node: TreeContent,
    prefix: String,
    child_prefix: String,
    source: &'a str,
}

impl Display for Asn1FormatterInternal<'_> {
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
                    let fmt = Asn1FormatterInternal {
                        depth: self.depth + 1,
                        tree: self.tree,
                        node,
                        prefix: self.child_prefix.clone() + "|-- ",
                        child_prefix: self.child_prefix.clone() + "|   ",
                        source: self.source,
                    };

                    write!(f, "{fmt}")?;
                }

                let fmt = Asn1FormatterInternal {
                    depth: self.depth + 1,
                    tree: self.tree,
                    node: *last,
                    prefix: self.child_prefix.clone() + "`-- ",
                    child_prefix: self.child_prefix.clone() + "    ",
                    source: self.source,
                };

                write!(f, "{fmt}")?;
            }
            TreeContent::Token(t) => writeln!(
                f,
                "{:?}: {:?}",
                t.kind,
                &self.source[t.offset..t.offset + t.length]
            )?,
        }

        Ok(())
    }
}

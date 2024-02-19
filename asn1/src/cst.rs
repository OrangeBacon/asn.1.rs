//! Representation of a parsed ASN.1 description file (NOT an encoded message)
//! This is a Concrete Syntax Tree (not an Abstract Syntax Tree), meaning all
//! tokens from the source file are included, such as brackets, comments, etc.
//! This is useful for formatting, codegen, IDE like features, but not as good
//! for analysis as the structure is much less strict than would be present for
//! an AST, therefore an AST is also implemented as a view over this CST (in
//! another module).

use std::{fmt::Display, iter::Peekable, ops::Range};

use crate::{
    compiler::SourceId,
    token::{Token, TokenKind},
};

/// A whole ASN.1 file including all modules
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Asn1 {
    /// ID of the node containing the root node of the tree
    pub root: AsnNodeId,

    /// Flattened representation of all data contained within the tree
    data: Vec<TreeContent>,

    /// ID of the source file the tree was created from
    id: SourceId,
}

/// ID representing a single ASN.1 CST node
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AsnNodeId(usize, SourceId);

/// Content of a tree
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TreeContent {
    /// Header of a nested node in the tree with a given kind
    Tree {
        tag: Asn1Tag,
        start: usize,
        count: usize,
    },

    /// A token from a given source file
    Token {
        /// The type of this token
        kind: TokenKind,

        /// The byte length of the source of the token in its source file.
        length: usize,

        /// Byte offset into the file that the token starts at.  The end location
        /// can be derived from this offset + the length of the value string.
        offset: usize,
    },
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

impl Asn1 {
    /// Create a CST from an externally constructed tree.
    pub fn new(id: SourceId, data: Vec<TreeContent>, root_idx: usize) -> Asn1 {
        Asn1 {
            root: AsnNodeId(root_idx, id),
            data,
            id,
        }
    }

    /// Create an iterator over the nested node contents of a tree node.  Returns
    /// `None` if the chosen node is a token node, not a tree node.
    pub fn iter_tree(&self, node: AsnNodeId) -> Option<CstIter> {
        debug_assert_eq!(node.1, self.id);

        match self.data[node.0] {
            TreeContent::Tree {
                tag, start, count, ..
            } => {
                let id = self.id;
                Some(CstIter {
                    tag,
                    range: (start..start + count).peekable(),
                    id,
                })
            }
            TreeContent::Token { .. } => None,
        }
    }

    /// Get the tag of a provided tree node.  Returns
    /// `None` if the chosen node is a token node, not a tree node.
    pub fn tree_tag(&self, node: AsnNodeId) -> Option<Asn1Tag> {
        debug_assert_eq!(node.1, self.id);

        match self.data[node.0] {
            TreeContent::Tree { tag, .. } => Some(tag),
            _ => None,
        }
    }

    /// Get a token from the tree
    pub fn token(&self, node: AsnNodeId) -> Option<Token> {
        debug_assert_eq!(node.1, self.id);

        match self.data[node.0] {
            TreeContent::Token {
                kind,
                length,
                offset,
            } => Some(Token {
                kind,
                length,
                offset,
                id: self.id,
            }),
            _ => None,
        }
    }
}

impl TreeContent {
    /// Construct a new tree node from a given token.  Does not check the source
    /// ID of the token.
    pub fn new(tok: Token) -> TreeContent {
        TreeContent::Token {
            kind: tok.kind,
            length: tok.length,
            offset: tok.offset,
        }
    }
}

/// Formatter for the CST of an asn1 file
pub(crate) struct Asn1Formatter<'a> {
    /// The tree to be formatted
    pub tree: &'a Asn1,

    /// The source text the tree was created from
    pub source: &'a str,
}

impl Display for Asn1Formatter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt = Asn1FormatterInternal {
            depth: 0,
            tree: self.tree,
            node: self.tree.data[self.tree.root.0],
            prefix: String::new(),
            child_prefix: String::new(),
            source: self.source,
        };

        write!(f, "{fmt}")
    }
}

/// Pretty printer for a nested tree
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
            TreeContent::Token {
                kind,
                length,
                offset,
            } => writeln!(f, "{:?}: {:?}", kind, &self.source[offset..offset + length])?,
        }

        Ok(())
    }
}

/// Iterator over CST Nodes
pub struct CstIter {
    /// The tag of the tree node that is being iterated over
    pub tag: Asn1Tag,

    /// The source iterator representing indexes into a cst
    range: Peekable<Range<usize>>,

    /// The id of the source file this iterator came from
    pub id: SourceId,
}

impl Iterator for CstIter {
    type Item = AsnNodeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.range.next().map(|e| AsnNodeId(e, self.id))
    }
}

impl CstIter {
    /// Try to get the next node ID without consuming it
    pub fn peek(&mut self) -> Option<AsnNodeId> {
        self.range.peek().map(|e| AsnNodeId(*e, self.id))
    }
}

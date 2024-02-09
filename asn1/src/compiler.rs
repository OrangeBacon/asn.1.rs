//! The primary interface to all the ASN.1 parsing, codegen, analysis, and other tools.

use crate::{
    cst::{Asn1, Asn1Formatter},
    lexer::Lexer,
    parser::{Parser, ParserError},
};

/// Store of all information relating to a whole ASN.1 specification, including
/// multiple files, analysis and code generation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct AsnCompiler {
    /// List of all included source files.
    sources: Vec<Source>,
}

/// Information relating to a single source file
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Source {
    /// File name and path.
    file_name: String,

    /// Source text of the file
    source: String,

    /// The concrete syntax tree of the file.
    tree: Asn1,
}

/// Reference to a single source file
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceId(usize);

impl AsnCompiler {
    /// Create a new compiler
    pub fn new() -> Self {
        Default::default()
    }

    /// Add a new file to the compiler
    pub fn add_file(&mut self, file_name: String, source: String) -> Result<SourceId, ParserError> {
        let id = SourceId(self.sources.len());

        let lexer = Lexer::new(id, &source);
        let tree = Parser::new(lexer).run()?;

        self.sources.push(Source {
            file_name,
            source,
            tree,
        });

        Ok(id)
    }

    /// Convert the CST of a file into a string
    pub fn print_cst(&self, file: SourceId) -> String {
        let source = &self.sources[file.0];
        Asn1Formatter {
            tree: &source.tree,
            source: &source.source,
        }
        .to_string()
    }
}

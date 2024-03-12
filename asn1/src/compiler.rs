//! The primary interface to all the ASN.1 parsing, codegen, analysis, and other tools.

use std::ops::{Deref, DerefMut};

use crate::{
    analysis::AnalysisContext,
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

    /// The enabled features.
    features: Features,
}

/// All features that can be enabled within the compiler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Features {
    /// Allow both upper and lowercase keywords.
    pub lowercase_keywords: bool,

    /// Allow non-ascii characters in identifiers
    pub unicode_identifiers: bool,

    /// Allow further whitespace characters
    pub unicode_whitespace: bool,
}

/// Information relating to a single source file
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Source {
    /// File name and path.
    pub(crate) file_name: String,

    /// Source text of the file
    pub(crate) source: String,

    /// The concrete syntax tree of the file.
    pub(crate) tree: Asn1,

    /// ID of the source
    pub(crate) id: SourceId,
}

/// Reference to a single source file
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceId(usize);

impl AsnCompiler {
    /// Create a new compiler
    pub fn new() -> Self {
        Default::default()
    }

    /// Add a new file to the compiler.  Will do some initial parsing, but will
    /// not run any analysis that is required to check that the source files
    /// are valid.
    pub fn add_file(&mut self, file_name: String, source: String) -> Result<SourceId, ParserError> {
        let id = SourceId(self.sources.len());

        let lexer = Lexer::new(id, &source, self.features);
        let tree = Parser::new(lexer).run()?;

        self.sources.push(Source {
            file_name,
            source,
            tree,
            id,
        });

        Ok(id)
    }

    /// Get an iterator over all source IDs
    pub(crate) fn all_sources(&self) -> impl Iterator<Item = SourceId> + '_ {
        self.sources.iter().map(|s| s.id)
    }

    /// Get the source associated with a source id
    #[inline]
    pub(crate) fn source(&self, file: SourceId) -> &Source {
        &self.sources[file.0]
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

    /// Run static analysis of all the provided source files.
    pub fn analysis(&mut self) -> AnalysisContext {
        AnalysisContext::new(self)
    }
}

impl Deref for AsnCompiler {
    type Target = Features;

    fn deref(&self) -> &Self::Target {
        &self.features
    }
}

impl DerefMut for AsnCompiler {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.features
    }
}

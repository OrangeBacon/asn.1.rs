mod error;
mod module;
mod parameterized;
mod reference;
mod type_or_value;
mod xml_value;

use crate::{
    cst::{Asn1, Asn1Tag, TreeContent},
    lexer::Lexer,
    token::{Token, TokenKind},
    util::CowVec,
};

pub use self::error::{ParserError, Result};

/// Parser for ASN.1 definition files
#[derive(Debug, Clone)]
pub struct Parser<'a> {
    /// Lexer to get tokens from a source file
    lexer: Lexer<'a>,

    /// The partial tree constructed by the parser
    result: Vec<TreeContent<'a>>,

    /// Temporary storage used when making the tree
    temp_result: Vec<TreeContent<'a>>,

    /// Data to finish constructing a partial cst in error cases
    error_nodes: Vec<TempVec>,

    /// Current recursion depth of the parser, as measured by start temp vec.
    depth: usize,
}

/// Helper for constructing cst tree nodes from the temp_result array in the error
/// case, when unwinding (through result) through the parser.
#[derive(Debug, Clone)]
struct TempVec {
    tag: Asn1Tag,
    offset: usize,
}

impl<'a> Parser<'a> {
    /// Create a new parser from a lexer
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self {
            lexer,
            result: vec![],
            temp_result: vec![],
            error_nodes: vec![],
            depth: 0,
        }
    }

    /// Run the parser to produce a set of ASN.1 definitions
    pub fn run(mut self) -> Result<Asn1<'a>> {
        self.start_temp_vec(Asn1Tag::Root)?;

        while !self.lexer.is_eof() {
            self.module_definition()?;
        }

        // handle comments at the end of the file after all meaningful tokens
        let _ = self.next(&[]);

        self.end_temp_vec(Asn1Tag::Root);
        let root = self.result.len();
        self.result.push(self.temp_result[0]);

        Ok(Asn1 {
            root,
            data: self.result,
        })
    }

    /// Consume a token of the given kind or return an error.  Ignores any comment tokens.
    /// If an empty list is given, returns any token.
    fn next(&mut self, kind: impl Into<CowVec<TokenKind>>) -> Result<Token<'a>> {
        self.peek(kind)?;

        loop {
            let tok = self.lexer.next_token()?;
            self.temp_result.push(TreeContent::Token(tok));

            if tok.kind != TokenKind::SingleComment && tok.kind != TokenKind::MultiComment {
                return Ok(tok);
            }
        }
    }

    /// Peek a token without consuming it or return an error if the token is not
    /// of one of the provided kinds. If an empty list is given, returns any token.
    fn peek(&mut self, kind: impl Into<CowVec<TokenKind>>) -> Result<Token<'a>> {
        let kind = kind.into();

        let peek = self.lexer.peek()?;

        if kind.contains(&peek.kind) || kind.is_empty() {
            Ok(peek)
        } else {
            Err(ParserError::Expected {
                kind,
                offset: peek.offset,
                file: peek.file,
            })
        }
    }

    /// Start an ast tree node with the given tag to describe the node
    fn start_temp_vec(&mut self, tag: Asn1Tag) -> Result {
        // TODO: make this an actual parameter, not a magic number I picked randomly
        if self.depth >= 100 {
            return Err(ParserError::ParserDepthExceeded {
                offset: self.lexer.offset(),
                file: self.lexer.file,
            });
        }

        self.error_nodes.push(TempVec {
            tag,
            offset: self.temp_result.len(),
        });

        Ok(())
    }

    /// End the most recent temporary vec.
    #[track_caller]
    fn end_temp_vec(&mut self, tag: Asn1Tag) {
        let end = self.error_nodes.pop().unwrap();

        debug_assert_eq!(tag, end.tag);

        let temp_start = end.offset;

        let start = self.result.len();
        let count = self.temp_result.len() - temp_start;

        self.result.extend(self.temp_result.drain(temp_start..));

        self.temp_result.push(TreeContent::Tree {
            tag: end.tag,
            start,
            count,
        })
    }
}

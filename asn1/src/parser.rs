mod module;
mod parameterized;
mod reference;
mod type_or_value;
mod xml_value;

use crate::{
    compiler::SourceId,
    cst::{Asn1, Asn1Tag, TreeContent},
    diagnostic::{Diagnostic, Label, Result},
    lexer::Lexer,
    token::{Token, TokenKind},
    util::CowVec,
    AsnCompiler,
};

/// Parser for ASN.1 definition files
#[derive(Debug)]
pub struct Parser<'a> {
    /// Lexer to get tokens from a source file
    lexer: Lexer<'a>,

    /// The partial tree constructed by the parser
    result: Vec<TreeContent>,

    /// Temporary storage used when making the tree
    temp_result: Vec<TreeContent>,

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

impl AsnCompiler {
    /// Create a new parser from a lexer
    pub fn parser<'a>(&'a mut self, id: SourceId, source: &'a str) -> Parser<'a> {
        let lexer = self.lexer(id, source);

        Parser {
            lexer,
            result: vec![],
            temp_result: vec![],
            error_nodes: vec![],
            depth: 0,
        }
    }
}

impl<'a> Parser<'a> {
    /// Run the parser to produce a set of ASN.1 definitions
    pub fn run(mut self) -> Result<Asn1> {
        self.start_temp_vec(Asn1Tag::Root)?;

        while !self.lexer.is_eof() {
            self.module_definition()?;
        }

        // handle comments at the end of the file after all meaningful tokens
        self.consume_comments();

        self.end_temp_vec(Asn1Tag::Root);
        let root = self.result.len();
        self.result.push(self.temp_result[0]);

        Ok(Asn1::new(self.lexer.id, self.result, root))
    }

    /// Consume a token of the given kind or return an error.  Ignores any comment tokens.
    /// If an empty list is given, returns any token.
    fn next(&mut self, kind: impl Into<CowVec<TokenKind>>) -> Result<Token> {
        self.peek(kind)?;

        loop {
            let tok = self.lexer.next_token()?;
            self.temp_result.push(TreeContent::new(tok));

            if tok.kind != TokenKind::SingleComment && tok.kind != TokenKind::MultiComment {
                return Ok(tok);
            }
        }
    }

    /// Peek a token without consuming it or return an error if the token is not
    /// of one of the provided kinds. If an empty list is given, returns any token.
    fn peek(&mut self, kind: impl Into<CowVec<TokenKind>>) -> Result<Token> {
        let kind = kind.into();

        let peek = self.lexer.peek()?;

        if kind.contains(&peek.kind) || kind.is_empty() {
            Ok(peek)
        } else {
            // Err(ParserError::Expected {
            //     kind,
            //     got: peek.kind,
            //     offset: peek.offset,
            //     id: peek.id,
            // })
            Err(Diagnostic::error("Asn::Parser::Syntax").name("Syntax Error"))
        }
    }

    /// Peek the next XML token from the lexer without consuming it
    fn peek_xml(&mut self, kind: &[TokenKind]) -> Result<Token> {
        let tok = self.lexer.peek_xml()?;

        if kind.contains(&tok.kind) || kind.is_empty() {
            Ok(tok)
        } else {
            // Err(ParserError::Expected {
            //     kind: kind.to_vec().into(),
            //     got: tok.kind,
            //     offset: tok.offset,
            //     id: tok.id,
            // })
            Err(Diagnostic::error("Asn::Parser::XmlSyntax").name("Syntax Error in XML Literal"))
        }
    }

    /// Get the next XML token from the lexer
    fn next_xml(&mut self, kind: &[TokenKind]) -> Result<Token> {
        let tok = self.lexer.next_xml()?;
        self.temp_result.push(TreeContent::new(tok));

        if kind.contains(&tok.kind) || kind.is_empty() {
            Ok(tok)
        } else {
            // Err(ParserError::Expected {
            //     kind: kind.to_vec().into(),
            //     got: tok.kind,
            //     offset: tok.offset,
            //     id: tok.id,
            // })
            Err(Diagnostic::error("Asn::Parser::XmlSyntax").name("Syntax Error in XML Literal"))
        }
    }

    /// Consume all comment tokens from the lexer
    fn consume_comments(&mut self) {
        while let Some(tok) = self.lexer.next_comment() {
            self.temp_result.push(TreeContent::new(tok));
        }
    }

    /// Start an ast tree node with the given tag to describe the node
    fn start_temp_vec(&mut self, tag: Asn1Tag) -> Result {
        // TODO: make this an actual parameter, not a magic number I picked randomly
        if self.depth >= 100 {
            // return Err(ParserError::ParserDepthExceeded {
            //     offset: self.lexer.offset(),
            //     id: self.lexer.id,
            // });
            let loc = self.lexer.offset();
            return Err(Diagnostic::error("Asn::Parser::Depth")
                .name("Parser recursion depth limit reached")
                .label("Try refactoring your code into multiple less-complex definitions")
                .label(
                    Label::new()
                        .source(self.lexer.id)
                        .loc(loc..loc)
                        .message("Limit reached here"),
                ));
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

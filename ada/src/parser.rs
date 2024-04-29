mod file;

use std::cell::Cell;

use crate::{
    cst::{Event, TreeKind},
    diagnostic::{Diagnostic, Result},
    token::{Token, TokenKind},
};

/// State used while parsing a file
pub struct Parser {
    /// All tokens within the source file
    tokens: Vec<Token>,

    /// Offset of the next token to be consumed
    position: usize,

    /// Amount of work allowed for the current token - used to prevent infinite
    /// loops where one token is never consumed.
    fuel: Cell<u8>,

    /// Amount of recursion depth remaining - used to prevent either infinite
    /// loops or stack overflows due to overly complicated inputs.
    recursion_fuel: Cell<u8>,

    /// All events created while parsing the source file.
    events: Vec<Event>,

    /// Any errors found during parsing
    errors: Vec<Diagnostic>,
}

/// The result of a successful parse
#[derive(Debug, Clone)]
pub struct ParseOutput {
    /// The tokens from the source file
    tokens: Vec<Token>,

    /// The events used to convert the tokens into a tree
    events: Vec<Event>,
}

/// Marker for creating tree events within the parser
struct MarkOpened {
    index: usize,
}

impl Parser {
    /// Construct a parser from a list of tokens.
    ///
    /// The parser is required to detect where character literals can be, so it
    /// can determine the difference between attributes and character literals,
    /// as in the following example:
    /// ```ada
    /// A : Character := Character'Val (32);     -- A is now a space
    /// B : Character := ' ';                    -- B is also a space
    /// S : String    := Character'(')')'Image;  -- an especially nice parsing exercise
    /// ```
    /// This requires detecting whitespace within the character literal, so no filtering
    /// should be done before the parser is called
    pub fn run(tokens: Vec<Token>) -> Result<ParseOutput, Vec<Diagnostic>> {
        let mut parser = Self {
            tokens,
            position: 0,
            fuel: Cell::new(255),
            recursion_fuel: Cell::new(255),
            events: vec![],
            errors: vec![],
        };

        file::file(&mut parser);

        if parser.errors.is_empty() {
            Ok(ParseOutput {
                tokens: parser.tokens,
                events: parser.events,
            })
        } else {
            Err(parser.errors)
        }
    }

    /// start a new nested tree node
    fn open(&mut self) -> MarkOpened {
        assert_ne!(self.recursion_fuel.get(), 0);
        self.recursion_fuel.set(self.recursion_fuel.get() - 1);

        let mark = MarkOpened {
            index: self.events.len(),
        };
        self.events.push(Event::Open {
            kind: TreeKind::ErrorTree,
        });
        mark
    }

    /// end a nested tree node
    fn close(&mut self, m: MarkOpened, kind: TreeKind) {
        self.recursion_fuel.set(self.recursion_fuel.get() + 1);

        self.events[m.index] = Event::Open { kind };
        self.events.push(Event::Close);
    }

    /// Consume the next token (non-whitespace)
    fn advance(&mut self) {
        assert!(!self.eof());
        self.fuel.set(255);

        loop {
            self.events.push(Event::Advance);
            self.position += 1;
            if self
                .tokens
                .get(self.position)
                .map(|t| is_not_trivia(&t))
                .unwrap_or(true)
            {
                break;
            }
        }
    }

    /// is the parser at the end of the file
    fn eof(&self) -> bool {
        self.position >= self.tokens.len()
    }

    /// Get the nth next token (non-whitespace)
    fn nth(&self, lookahead: usize) -> TokenKind {
        assert_ne!(self.fuel.get(), 0);
        self.fuel.set(self.fuel.get() - 1);

        self.tokens
            .get(self.position + lookahead..)
            .map(|t| t.iter().filter(is_not_trivia))
            .and_then(|mut i| i.next())
            .map(|t| t.kind)
            .unwrap_or(TokenKind::Eof)
    }

    /// is the next token of the provided (non-whitespace) token kind?
    fn at(&self, kind: TokenKind) -> bool {
        self.nth(0) == kind
    }

    /// Try to consume a token of the given (non-whitespace) kind
    fn eat(&mut self, kind: TokenKind) -> bool {
        if self.at(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Consume a token of a given (non-whitespace) kind, otherwise error.
    fn expect(&mut self, kind: TokenKind) {
        if self.eat(kind) {
            return;
        }

        self.errors
            .push(Diagnostic::new(format!("Expected {kind:?}")));
    }

    /// Add the next (non-whitespace) token to an error tree and consume it
    fn advance_with_error(&mut self, message: impl Into<String>) {
        let m = self.open();

        self.errors.push(Diagnostic::new(message));
        self.advance();

        self.close(m, TreeKind::ErrorTree);
    }
}

/// is a token a trivia token?
fn is_not_trivia(token: &&Token) -> bool {
    !(TokenKind::Comment | TokenKind::Whitespace).contains(token.kind)
}

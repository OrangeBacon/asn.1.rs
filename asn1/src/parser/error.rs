use crate::{lexer::LexerError, token::TokenKind, util::CowVec};

/// Any error that can be emitted by the parser
#[derive(Debug, Clone)]
pub enum ParserError {
    /// Unable to lex one of the token kinds at a given offset into a file
    Expected {
        kind: CowVec<TokenKind>,
        got: TokenKind,
        offset: usize,
        file: usize,
    },

    /// Recursion depth limit reached in the parser (try to avoid stack overflow)
    ParserDepthExceeded { offset: usize, file: usize },

    /// An error occurred within the lexer
    LexerError(LexerError),

    /// An error occurred while trying to parse the given type or value command
    TypeValueError {
        subsequent: Vec<TokenKind>,
        alternative: Vec<TokenKind>,
        got: TokenKind,
        offset: usize,
        file: usize,
    },
}

pub type Result<T = (), E = ParserError> = std::result::Result<T, E>;

impl From<LexerError> for ParserError {
    fn from(value: LexerError) -> Self {
        ParserError::LexerError(value)
    }
}

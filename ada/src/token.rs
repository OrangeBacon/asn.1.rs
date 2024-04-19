
/// All possible types of token, includes invalid tokens e.g. error and EOF.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TokenKind {
    Error,
    Eof,
}

/// A single token from a source file
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Token {
    /// The type of the token
    pub kind: TokenKind,

    /// The start byte index into the source file
    pub start: u32,

    /// The end byte index into the source file (exclusive upper bound)
    pub end: u32,
}

/// The kind of a lexed token
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum TokenKind {
    // Comments
    SingleComment,
    MultiComment,

    // Single Character tokens
    LeftCurly,
    RightCurly,
    Less,
    Greater,
    Comma,
    Dot,
    ForwardSlash,
    LeftParen,
    RightParen,
    LeftSquare,
    RightSquare,
    Hyphen,
    Colon,
    Equals,
    DoubleQuote,
    SingleQuote,
    SemiColon,
    At,
    Pipe,
    Exclamation,
    Caret,

    // Compound Tokens
    Identifier,

    // Errors
    Unrecognised,
    NonTerminatedComment,
}

/// Data relating to a single lexed token
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Token<'a> {
    /// The type of this token
    pub(crate) kind: TokenKind,

    /// The string value of the token, will be a valid string for the token kind
    /// so it can be parsed further, e.g. into a number.
    pub(crate) value: &'a str,

    /// Byte offset into the file that the token starts at.  The end location
    /// can be derived from this offset + the length of the value string.
    pub(crate) offset: usize,

    /// The file ID of the file the token was lexed from
    pub(crate) file: usize,
}

/// Data relating to a single lexed token, owning the string value of the token,
/// rather than holding a reference to the source.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenBuffer {
    /// The type of this token
    pub(crate) kind: TokenKind,

    /// The string value of the token, will be a valid string for the token kind
    /// so it can be parsed further, e.g. into a number.
    pub(crate) value: String,

    /// Byte offset into the file that the token starts at.  The end location
    /// can be derived from this offset + the length of the value string.
    pub(crate) offset: usize,

    /// The file ID of the file the token was lexed from
    pub(crate) file: usize,
}

impl Token<'_> {
    /// Convert a token to one that owns its value
    pub fn to_owned(&self) -> TokenBuffer {
        TokenBuffer {
            kind: self.kind,
            value: self.value.to_string(),
            offset: self.offset,
            file: self.file,
        }
    }
}

impl TokenBuffer {
    /// Get a non-owning reference token to this owned token
    pub fn as_token(&self) -> Token<'_> {
        Token {
            kind: self.kind,
            value: &self.value,
            offset: self.offset,
            file: self.file,
        }
    }
}

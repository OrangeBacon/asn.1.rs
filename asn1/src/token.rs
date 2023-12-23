/// The kind of a lexed token
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TokenKind {
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
    SemiColon,
    At,
    Pipe,
    Exclamation,
    Caret,

    // Compound Tokens
    Identifier,
    TypeReference,
    ValueReference,
    Assignment,

    // Keywords
    KwBoolean,
    KwTrue,
    KwFalse,

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

/// String/Enum mapping for keywords
pub const KEYWORD_DATA: [(&str, TokenKind); 3] = [
    ("BOOLEAN", TokenKind::KwBoolean),
    ("TRUE", TokenKind::KwTrue),
    ("FALSE", TokenKind::KwFalse),
    // "ABSENT",
    // "ABSTRACT-SYNTAX",
    // "ALL",
    // "APPLICATION",
    // "AUTOMATIC",
    // "BEGIN",
    // "BIT",
    // "BMPString",
    // "BY",
    // "CHARACTER",
    // "CHOICE",
    // "CLASS",
    // "COMPONENT",
    // "COMPONENTS",
    // "CONSTRAINED",
    // "CONTAINING",
    // "DATE",
    // "DATE-TIME",
    // "DEFAULT",
    // "DEFINITIONS",
    // "DURATION",
    // "EMBEDDED",
    // "ENCODED",
    // "ENCODING-CONTROL",
    // "END",
    // "ENUMERATED",
    // "EXCEPT",
    // "EXPLICIT",
    // "EXPORTS",
    // "EXTENSIBILITY",
    // "EXTERNAL",
    // "FROM",
    // "GeneralizedTime",
    // "GeneralString",
    // "IA5String",
    // "IDENTIFIER",
    // "IMPLICIT",
    // "IMPLIED",
    // "IMPORTS",
    // "INCLUDES",
    // "INSTANCE",
    // "INSTRUCTIONS",
    // "INTEGER",
    // "INTERSECTION",
    // "ISO646String",
    // "MAX",
    // "MIN",
    // "MINUS-INFINITY",
    // "NOT-A-NUMBER",
    // "NULL",
    // "NumericString",
    // "OBJECT",
    // "ObjectDescriptor",
    // "OCTET",
    // "OF",
    // "OID-IRI",
    // "OPTIONAL",
    // "PATTERN",
    // "PDV",
    // "PLUS-INFINITY",
    // "PRESENT",
    // "PrintableString",
    // "PRIVATE",
    // "REAL",
    // "RELATIVE-OID",
    // "RELATIVE-OID-IRI",
    // "SEQUENCE",
    // "SET",
    // "SETTINGS",
    // "SIZE",
    // "STRING",
    // "SYNTAX",
    // "T61String",
    // "TAGS",
    // "TeletexString",
    // "TIME",
    // "TIME-OF-DAY",
    // "TYPE-IDENTIFIER",
    // "UNION",
    // "UNIQUE",
    // "UNIVERSAL",
    // "UniversalString",
    // "UTCTime",
    // "UTF8String",
    // "VideotexString",
    // "VisibleString",
    // "WITH",
];

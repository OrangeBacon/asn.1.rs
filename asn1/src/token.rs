/// The kind of a lexed token
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TokenKind {
    // Comments
    SingleComment,
    MultiComment,

    // Single character tokens
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
    SemiColon,
    At,
    Pipe,
    Exclamation,
    Caret,

    // Multi-character tokens
    Assignment,

    // Compound tokens
    Identifier,
    TypeReference,
    ModuleReference,
    EncodingReference,
    ValueReference,
    IntegerUnicodeLabel,
    NonIntegerUnicodeLabel,
    Number,

    // Keywords
    // "ABSENT",
    // "ABSTRACT-SYNTAX",
    // "ALL",
    // "APPLICATION",
    KwAutomatic,
    KwBegin,
    // "BIT",
    // "BMPString",
    KwBoolean,
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
    KwDefinitions,
    // "DURATION",
    // "EMBEDDED",
    // "ENCODED",
    // "ENCODING-CONTROL",
    KwEnd,
    // "ENUMERATED",
    // "EXCEPT",
    KwExplicit,
    // "EXPORTS",
    KwExtensibility,
    // "EXTERNAL",
    KwFalse,
    // "FROM",
    // "GeneralizedTime",
    // "GeneralString",
    // "IA5String",
    // "IDENTIFIER",
    KwImplicit,
    KwImplied,
    // "IMPORTS",
    // "INCLUDES",
    // "INSTANCE",
    KwInstructions,
    // "INTEGER",
    // "INTERSECTION",
    // "ISO646String",
    // "MAX",
    // "MIN",
    // "MINUS-INFINITY",
    // "NOT-A-NUMBER",
    KwNull,
    // "NumericString",
    // "OBJECT",
    // "ObjectDescriptor",
    // "OCTET",
    // "OF",
    KwOidIri,
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
    KwTags,
    // "TeletexString",
    // "TIME",
    // "TIME-OF-DAY",
    KwTrue,
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
pub const KEYWORD_DATA: [(&str, TokenKind); 15] = [
    // "ABSENT",
    // "ABSTRACT-SYNTAX",
    // "ALL",
    // "APPLICATION",
    ("AUTOMATIC", TokenKind::KwAutomatic),
    ("BEGIN", TokenKind::KwBegin),
    // "BIT",
    // "BMPString",
    ("BOOLEAN", TokenKind::KwBoolean),
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
    ("DEFINITIONS", TokenKind::KwDefinitions),
    // "DURATION",
    // "EMBEDDED",
    // "ENCODED",
    // "ENCODING-CONTROL",
    ("END", TokenKind::KwEnd),
    // "ENUMERATED",
    // "EXCEPT",
    ("EXPLICIT", TokenKind::KwExplicit),
    // "EXPORTS",
    ("EXTENSIBILITY", TokenKind::KwExtensibility),
    // "EXTERNAL",
    ("FALSE", TokenKind::KwFalse),
    // "FROM",
    // "GeneralizedTime",
    // "GeneralString",
    // "IA5String",
    // "IDENTIFIER",
    ("IMPLICIT", TokenKind::KwImplicit),
    ("IMPLIED", TokenKind::KwImplied),
    // "IMPORTS",
    // "INCLUDES",
    // "INSTANCE",
    ("INSTRUCTIONS", TokenKind::KwInstructions),
    // "INTEGER",
    // "INTERSECTION",
    // "ISO646String",
    // "MAX",
    // "MIN",
    // "MINUS-INFINITY",
    // "NOT-A-NUMBER",
    ("NULL", TokenKind::KwNull),
    // "NumericString",
    // "OBJECT",
    // "ObjectDescriptor",
    // "OCTET",
    // "OF",
    ("OID-IRI", TokenKind::KwOidIri),
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
    ("TAGS", TokenKind::KwTags),
    // "TeletexString",
    // "TIME",
    // "TIME-OF-DAY",
    ("TRUE", TokenKind::KwTrue),
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

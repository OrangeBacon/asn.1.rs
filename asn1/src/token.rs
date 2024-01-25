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
    SemiColon,
    At,
    Pipe,
    Exclamation,
    Caret,
    Underscore,

    // Multi-character tokens
    Assignment,
    XMLEndTag,
    XMLSingleTagEnd,
    Ellipsis,

    // Compound tokens
    ValueRefOrIdent,
    TypeOrModuleRef,
    IntegerUnicodeLabel,
    NonIntegerUnicodeLabel,
    Number,
    XMLAsn1TypeName,
    XMLBoolNumber,
    CString,
    BHString,
    Field,

    // Keywords
    // "ABSENT",
    KwAbstractSyntax,
    KwAll,
    KwApplication,
    KwAutomatic,
    KwBegin,
    KwBit,
    KwBmpString,
    KwBoolean,
    // "BY",
    KwCharacter,
    // "CHOICE",
    // "CLASS",
    // "COMPONENT",
    // "COMPONENTS",
    // "CONSTRAINED",
    KwContaining,
    KwDate,
    KwDateTime,
    // "DEFAULT",
    KwDefinitions,
    KwDuration,
    KwEmbedded,
    // "ENCODED",
    // "ENCODING-CONTROL",
    KwEnd,
    KwEnumerated,
    // "EXCEPT",
    KwExplicit,
    KwExports,
    KwExtensibility,
    KwExternal,
    KwFalse,
    KwFrom,
    KwGeneralizedTime,
    KwGeneralString,
    KwGraphicString,
    KwIA5String,
    KwIdentifier,
    KwImplicit,
    KwImplied,
    KwImports,
    // "INCLUDES",
    KwInstance,
    KwInstructions,
    KwInteger,
    // "INTERSECTION",
    KwISO64String,
    // "MAX",
    // "MIN",
    KwMinusInfinity,
    KwNotANumber,
    KwNull,
    KwNumericString,
    KwObject,
    KwObjectDescriptor,
    KwOctet,
    KwOf,
    KwOidIri,
    // "OPTIONAL",
    // "PATTERN",
    KwPDV,
    KwPlusInfinity,
    // "PRESENT",
    KwPrintableString,
    KwPrivate,
    KwReal,
    KwRelativeOid,
    KwRelativeOidIri,
    // "SEQUENCE",
    // "SET",
    // "SETTINGS",
    // "SIZE",
    KwString,
    // "SYNTAX",
    KwT61String,
    KwTags,
    KwTeletexString,
    KwTime,
    KwTimeOfDay,
    KwTrue,
    KwTypeIdentifier,
    // "UNION",
    // "UNIQUE",
    KwUniversal,
    KwUniversalString,
    KwUTCTime,
    KwUTF8String,
    KwVideotexString,
    KwVisibleString,
    KwWith,
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

/// String/Enum mapping for keywords
pub const KEYWORD_DATA: [(&str, TokenKind); 66] = [
    // "ABSENT",
    ("ABSTRACT-SYNTAX", TokenKind::KwAbstractSyntax),
    ("ALL", TokenKind::KwAll),
    ("APPLICATION", TokenKind::KwApplication),
    ("AUTOMATIC", TokenKind::KwAutomatic),
    ("BEGIN", TokenKind::KwBegin),
    ("BIT", TokenKind::KwBit),
    ("BMPString", TokenKind::KwBmpString),
    ("BOOLEAN", TokenKind::KwBoolean),
    // "BY",
    ("CHARACTER", TokenKind::KwCharacter),
    // "CHOICE",
    // "CLASS",
    // "COMPONENT",
    // "COMPONENTS",
    // "CONSTRAINED",
    ("CONTAINING", TokenKind::KwContaining),
    ("DATE", TokenKind::KwDate),
    ("DATE-TIME", TokenKind::KwDateTime),
    // "DEFAULT",
    ("DEFINITIONS", TokenKind::KwDefinitions),
    ("DURATION", TokenKind::KwDuration),
    ("EMBEDDED", TokenKind::KwEmbedded),
    // "ENCODED",
    // "ENCODING-CONTROL",
    ("END", TokenKind::KwEnd),
    ("ENUMERATED", TokenKind::KwEnumerated),
    // "EXCEPT",
    ("EXPLICIT", TokenKind::KwExplicit),
    ("EXPORTS", TokenKind::KwExports),
    ("EXTENSIBILITY", TokenKind::KwExtensibility),
    ("EXTERNAL", TokenKind::KwExternal),
    ("FALSE", TokenKind::KwFalse),
    ("FROM", TokenKind::KwFrom),
    ("GeneralizedTime", TokenKind::KwGeneralizedTime),
    ("GeneralString", TokenKind::KwGeneralString),
    ("GraphicString", TokenKind::KwGraphicString),
    ("IA5String", TokenKind::KwIA5String),
    ("IDENTIFIER", TokenKind::KwIdentifier),
    ("IMPLICIT", TokenKind::KwImplicit),
    ("IMPLIED", TokenKind::KwImplied),
    ("IMPORTS", TokenKind::KwImports),
    // "INCLUDES",
    ("INSTANCE", TokenKind::KwInstance),
    ("INSTRUCTIONS", TokenKind::KwInstructions),
    ("INTEGER", TokenKind::KwInteger),
    // "INTERSECTION",
    ("ISO646String", TokenKind::KwISO64String),
    // "MAX",
    // "MIN",
    ("MINUS-INFINITY", TokenKind::KwMinusInfinity),
    ("NOT-A-NUMBER", TokenKind::KwNotANumber),
    ("NULL", TokenKind::KwNull),
    ("NumericString", TokenKind::KwNumericString),
    ("OBJECT", TokenKind::KwObject),
    ("ObjectDescriptor", TokenKind::KwObjectDescriptor),
    ("OCTET", TokenKind::KwOctet),
    ("OF", TokenKind::KwOf),
    ("OID-IRI", TokenKind::KwOidIri),
    // "OPTIONAL",
    // "PATTERN",
    ("PDV", TokenKind::KwPDV),
    ("PLUS-INFINITY", TokenKind::KwPlusInfinity),
    // "PRESENT",
    ("PrintableString", TokenKind::KwPrintableString),
    ("PRIVATE", TokenKind::KwPrivate),
    ("REAL", TokenKind::KwReal),
    ("RELATIVE-OID", TokenKind::KwRelativeOid),
    ("RELATIVE-OID-IRI", TokenKind::KwRelativeOidIri),
    // "SEQUENCE",
    // "SET",
    // "SETTINGS",
    // "SIZE",
    ("STRING", TokenKind::KwString),
    // "SYNTAX",
    ("T61String", TokenKind::KwT61String),
    ("TAGS", TokenKind::KwTags),
    ("TeletexString", TokenKind::KwTeletexString),
    ("TIME", TokenKind::KwTime),
    ("TIME-OF-DAY", TokenKind::KwTimeOfDay),
    ("TRUE", TokenKind::KwTrue),
    ("TYPE-IDENTIFIER", TokenKind::KwTypeIdentifier),
    // "UNION",
    // "UNIQUE",
    ("UNIVERSAL", TokenKind::KwUniversal),
    ("UniversalString", TokenKind::KwUniversalString),
    ("UTCTime", TokenKind::KwUTCTime),
    ("UTF8String", TokenKind::KwUTF8String),
    ("VideotexString", TokenKind::KwVideotexString),
    ("VisibleString", TokenKind::KwVisibleString),
    ("WITH", TokenKind::KwWith),
];

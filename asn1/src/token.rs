use crate::compiler::SourceId;

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
    VersionOpen,
    VersionClose,

    // Compound tokens
    ValueRefOrIdent,
    TypeOrModuleRef,
    Number,
    CString,
    BHString,
    XMLData,
    ValueField,
    TypeField,

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
    KwChoice,
    KwClass,
    // "COMPONENT",
    KwComponents,
    // "CONSTRAINED",
    KwContaining,
    KwDate,
    KwDateTime,
    KwDefault,
    KwDefinitions,
    KwDuration,
    KwEmbedded,
    // "ENCODED",
    KwEncodingControl,
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
    KwOptional,
    // "PATTERN",
    KwPDV,
    KwPlusInfinity,
    // "PRESENT",
    KwPrintableString,
    KwPrivate,
    KwReal,
    KwRelativeOid,
    KwRelativeOidIri,
    KwSequence,
    KwSet,
    // "SETTINGS",
    // "SIZE",
    KwString,
    KwSyntax,
    KwT61String,
    KwTags,
    KwTeletexString,
    KwTime,
    KwTimeOfDay,
    KwTrue,
    KwTypeIdentifier,
    // "UNION",
    KwUnique,
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
pub struct Token {
    /// The type of this token
    pub(crate) kind: TokenKind,

    /// The byte length of the source of the token in its source file.
    pub(crate) length: u32,

    /// Byte offset into the file that the token starts at.  The end location
    /// can be derived from this offset + the length of the value string.
    pub(crate) offset: usize,

    /// The file ID of the file the token was lexed from
    pub(crate) id: SourceId,
}

/// String/Enum mapping for keywords.  Contains both the normal and lowercase versions of the data.
pub const KEYWORD_DATA: [(&str, TokenKind, &str); 76] = [
    // "ABSENT",
    (
        "ABSTRACT-SYNTAX",
        TokenKind::KwAbstractSyntax,
        "AbstractSyntax",
    ),
    ("ALL", TokenKind::KwAll, "all"),
    ("APPLICATION", TokenKind::KwApplication, "application"),
    ("AUTOMATIC", TokenKind::KwAutomatic, "automatic"),
    ("BEGIN", TokenKind::KwBegin, "begin"),
    ("BIT", TokenKind::KwBit, "bit"),
    ("BMPString", TokenKind::KwBmpString, "BMPString"),
    ("BOOLEAN", TokenKind::KwBoolean, "boolean"),
    // "BY",
    ("CHARACTER", TokenKind::KwCharacter, "character"),
    ("CHOICE", TokenKind::KwChoice, "choice"),
    ("CLASS", TokenKind::KwClass, "class"),
    // "COMPONENT",
    ("COMPONENTS", TokenKind::KwComponents, "components"),
    // "CONSTRAINED",
    ("CONTAINING", TokenKind::KwContaining, "containing"),
    ("DATE", TokenKind::KwDate, "Date"),
    ("DATE-TIME", TokenKind::KwDateTime, "DateTime"),
    ("DEFAULT", TokenKind::KwDefault, "default"),
    ("DEFINITIONS", TokenKind::KwDefinitions, "definitions"),
    ("DURATION", TokenKind::KwDuration, "duration"),
    ("EMBEDDED", TokenKind::KwEmbedded, "embedded"),
    // "ENCODED",
    (
        "ENCODING-CONTROL",
        TokenKind::KwEncodingControl,
        "encoding-control",
    ),
    ("END", TokenKind::KwEnd, "end"),
    ("ENUMERATED", TokenKind::KwEnumerated, "enumerated"),
    // "EXCEPT",
    ("EXPLICIT", TokenKind::KwExplicit, "explicit"),
    ("EXPORTS", TokenKind::KwExports, "exports"),
    ("EXTENSIBILITY", TokenKind::KwExtensibility, "extensibility"),
    ("EXTERNAL", TokenKind::KwExternal, "external"),
    ("FALSE", TokenKind::KwFalse, "false"),
    ("FROM", TokenKind::KwFrom, "from"),
    (
        "GeneralizedTime",
        TokenKind::KwGeneralizedTime,
        "GeneralizedTime",
    ),
    ("GeneralString", TokenKind::KwGeneralString, "GeneralString"),
    ("GraphicString", TokenKind::KwGraphicString, "GraphicString"),
    ("IA5String", TokenKind::KwIA5String, "IA5String"),
    ("IDENTIFIER", TokenKind::KwIdentifier, "identifier"),
    ("IMPLICIT", TokenKind::KwImplicit, "implicit"),
    ("IMPLIED", TokenKind::KwImplied, "implied"),
    ("IMPORTS", TokenKind::KwImports, "imports"),
    // "INCLUDES",
    ("INSTANCE", TokenKind::KwInstance, "instance"),
    ("INSTRUCTIONS", TokenKind::KwInstructions, "instructions"),
    ("INTEGER", TokenKind::KwInteger, "integer"),
    // "INTERSECTION",
    ("ISO646String", TokenKind::KwISO64String, "ISO646String"),
    // "MAX",
    // "MIN",
    (
        "MINUS-INFINITY",
        TokenKind::KwMinusInfinity,
        "minus-infinity",
    ),
    ("NOT-A-NUMBER", TokenKind::KwNotANumber, "not-a-number"),
    ("NULL", TokenKind::KwNull, "null"),
    ("NumericString", TokenKind::KwNumericString, "NumericString"),
    ("OBJECT", TokenKind::KwObject, "object"),
    (
        "ObjectDescriptor",
        TokenKind::KwObjectDescriptor,
        "ObjectDescriptor",
    ),
    ("OCTET", TokenKind::KwOctet, "octet"),
    ("OF", TokenKind::KwOf, "of"),
    ("OID-IRI", TokenKind::KwOidIri, "oid-iri"),
    ("OPTIONAL", TokenKind::KwOptional, "optional"),
    // "PATTERN",
    ("PDV", TokenKind::KwPDV, "pdv"),
    ("PLUS-INFINITY", TokenKind::KwPlusInfinity, "plus-infinity"),
    // "PRESENT",
    (
        "PrintableString",
        TokenKind::KwPrintableString,
        "PrintableString",
    ),
    ("PRIVATE", TokenKind::KwPrivate, "private"),
    ("REAL", TokenKind::KwReal, "real"),
    ("RELATIVE-OID", TokenKind::KwRelativeOid, "relative-oid"),
    (
        "RELATIVE-OID-IRI",
        TokenKind::KwRelativeOidIri,
        "relative-oid-iri",
    ),
    ("SEQUENCE", TokenKind::KwSequence, "sequence"),
    ("SET", TokenKind::KwSet, "set"),
    // "SETTINGS",
    // "SIZE",
    ("STRING", TokenKind::KwString, "string"),
    ("SYNTAX", TokenKind::KwSyntax, "syntax"),
    ("T61String", TokenKind::KwT61String, "T61String"),
    ("TAGS", TokenKind::KwTags, "tags"),
    ("TeletexString", TokenKind::KwTeletexString, "TeletexString"),
    ("TIME", TokenKind::KwTime, "Time"),
    ("TIME-OF-DAY", TokenKind::KwTimeOfDay, "TimeOfDay"),
    ("TRUE", TokenKind::KwTrue, "true"),
    (
        "TYPE-IDENTIFIER",
        TokenKind::KwTypeIdentifier,
        "TypeIdentifier",
    ),
    // "UNION",
    ("UNIQUE", TokenKind::KwUnique, "unique"),
    ("UNIVERSAL", TokenKind::KwUniversal, "universal"),
    (
        "UniversalString",
        TokenKind::KwUniversalString,
        "UniversalString",
    ),
    ("UTCTime", TokenKind::KwUTCTime, "UTCTime"),
    ("UTF8String", TokenKind::KwUTF8String, "UTF8String"),
    (
        "VideotexString",
        TokenKind::KwVideotexString,
        "VideotexString",
    ),
    ("VisibleString", TokenKind::KwVisibleString, "VisibleString"),
    ("WITH", TokenKind::KwWith, "with"),
];

/// All possible types of token, includes invalid tokens e.g. error and EOF.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TokenKind {
    /// The source file was not in NFC form, so just emit one big error token
    NfcError,

    /// A given character is not valid in a source file
    UnicodeError,

    /// A given character is not valid in a source file outside of a comment
    UnicodeNotCommentError,

    /// The next character was not recognised as the start of any token
    Error,

    /// End of file
    Eof,

    Whitespace,
    Ampersand,
    Apostrophe,
    LParen,
    RParen,
    Star,
    Plus,
    Comma,
    Hyphen,
    Dot,
    Slash,
    Colon,
    SemiColon,
    LessThan,
    Equals,
    GreaterThan,
    At,
    LSquare,
    RSquare,
    VerticalBar,

    Arrow,
    DoubleDot,
    DoubleStar,
    ColonEquals,
    SlashEquals,
    GreaterEquals,
    LessEquals,
    LessLess,
    GreaterGreater,
    Box,

    Identifier,
    DecimalNumber,
    BasedNumber,
    Character,
    String,
    Comment,

    KwAbort,
    KwAbs,
    KwAbstract,
    KwAccept,
    KwAccess,
    KwAliased,
    KwAll,
    KwAnd,
    KwArray,
    KwAt,
    KwBegin,
    KwBody,
    KwCase,
    KwConstant,
    KwDeclare,
    KwDelay,
    KwDelta,
    KwDigits,
    KwDo,
    KwElse,
    KwElsif,
    KwEnd,
    KwEntry,
    KwException,
    KwExit,
    KwFor,
    KwFunction,
    KwGeneric,
    KwGoto,
    KwIf,
    KwIn,
    KwInterface,
    KwIs,
    KwLimited,
    KwLoop,
    KwMod,
    KwNew,
    KwNot,
    KwNull,
    KwOf,
    KwOr,
    KwOthers,
    KwOut,
    KwOverriding,
    KwPackage,
    KwParallel,
    KwPragma,
    KwPrivate,
    KwProcedure,
    KwProtected,
    KwRaise,
    KwRange,
    KwRecord,
    KwRem,
    KwRenames,
    KwRequeue,
    KwReturn,
    KwReverse,
    KwSelect,
    KwSeparate,
    KwSome,
    KwSubtype,
    KwSynchronized,
    KwTagged,
    KwTask,
    KwTerminate,
    KwThen,
    KwType,
    KwUntil,
    KwUse,
    KwWhen,
    KwWhile,
    KwWith,
    KwXor,
}

/// A single token from a source file
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Token {
    /// The type of the token
    pub kind: TokenKind,

    /// The start byte index into the source file
    pub start: usize,

    /// The end byte index into the source file (exclusive upper bound).  Note that
    /// this might equal the start, indicating that the token has a length of 0
    /// bytes (should only happen on EOF)
    pub end: usize,
}

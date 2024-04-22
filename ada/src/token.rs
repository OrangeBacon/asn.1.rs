use std::ops::BitOr;

/// All possible types of token, includes invalid tokens e.g. error and EOF.
// Note: A maximum of 128 tokens are supported (as tokens are put into bit sets)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum TokenKind {
    /// The source file was not in NFC form, so just emit one big error token
    NfcError,

    /// A given character is not valid in a source file
    UnicodeError,

    /// A given character is not valid in a source file outside of a comment
    UnicodeNotCommentError,

    /// An identifier was parsed, that was later shown to be invalid,
    IdentifierError,

    /// The next character was not recognised as the start of any token
    Error,

    /// End of file
    Eof,

    Whitespace,
    Comment,

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

/// A Bitset of all possible token kinds, used for fast token kind matching
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenKindFlags(u128);

impl BitOr<TokenKind> for TokenKind {
    type Output = TokenKindFlags;

    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self::Output {
        let a = 1 << self as u8;
        let b = 1 << rhs as u8;

        TokenKindFlags(a | b)
    }
}

impl BitOr<TokenKind> for TokenKindFlags {
    type Output = TokenKindFlags;

    #[inline(always)]
    fn bitor(self, rhs: TokenKind) -> Self::Output {
        TokenKindFlags(self.0 | (1 << rhs as u8))
    }
}

impl TokenKindFlags {
    /// Does this set of flags contain a given flag
    pub fn contains(self, k: TokenKind) -> bool {
        self.0 & (1 << k as u8) != 0
    }
}

/// Mapping between the text of a keyword and its token kind
pub static KEYWORDS: &[(&str, TokenKind)] = &[
    ("abort", TokenKind::KwAbort),
    ("abs", TokenKind::KwAbs),
    ("abstract", TokenKind::KwAbstract),
    ("accept", TokenKind::KwAccept),
    ("access", TokenKind::KwAccess),
    ("aliased", TokenKind::KwAliased),
    ("all", TokenKind::KwAll),
    ("and", TokenKind::KwAnd),
    ("array", TokenKind::KwArray),
    ("at", TokenKind::KwAt),
    ("begin", TokenKind::KwBegin),
    ("body", TokenKind::KwBody),
    ("case", TokenKind::KwCase),
    ("constant", TokenKind::KwConstant),
    ("declare", TokenKind::KwDeclare),
    ("delay", TokenKind::KwDelay),
    ("delta", TokenKind::KwDelta),
    ("digits", TokenKind::KwDigits),
    ("do", TokenKind::KwDo),
    ("else", TokenKind::KwElse),
    ("elsif", TokenKind::KwElsif),
    ("end", TokenKind::KwEnd),
    ("entry", TokenKind::KwEntry),
    ("exception", TokenKind::KwException),
    ("exit", TokenKind::KwExit),
    ("for", TokenKind::KwFor),
    ("function", TokenKind::KwFunction),
    ("generic", TokenKind::KwGeneric),
    ("goto", TokenKind::KwGoto),
    ("if", TokenKind::KwIf),
    ("in", TokenKind::KwIn),
    ("interface", TokenKind::KwInterface),
    ("is", TokenKind::KwIs),
    ("limited", TokenKind::KwLimited),
    ("loop", TokenKind::KwLoop),
    ("mod", TokenKind::KwMod),
    ("new", TokenKind::KwNew),
    ("not", TokenKind::KwNot),
    ("null", TokenKind::KwNull),
    ("of", TokenKind::KwOf),
    ("or", TokenKind::KwOr),
    ("others", TokenKind::KwOthers),
    ("out", TokenKind::KwOut),
    ("overriding", TokenKind::KwOverriding),
    ("package", TokenKind::KwPackage),
    ("parallel", TokenKind::KwParallel),
    ("pragma", TokenKind::KwPragma),
    ("private", TokenKind::KwPrivate),
    ("procedure", TokenKind::KwProcedure),
    ("protected", TokenKind::KwProtected),
    ("raise", TokenKind::KwRaise),
    ("range", TokenKind::KwRange),
    ("record", TokenKind::KwRecord),
    ("rem", TokenKind::KwRem),
    ("renames", TokenKind::KwRenames),
    ("requeue", TokenKind::KwRequeue),
    ("return", TokenKind::KwReturn),
    ("reverse", TokenKind::KwReverse),
    ("select", TokenKind::KwSelect),
    ("separate", TokenKind::KwSeparate),
    ("some", TokenKind::KwSome),
    ("subtype", TokenKind::KwSubtype),
    ("synchronized", TokenKind::KwSynchronized),
    ("tagged", TokenKind::KwTagged),
    ("task", TokenKind::KwTask),
    ("terminate", TokenKind::KwTerminate),
    ("then", TokenKind::KwThen),
    ("type", TokenKind::KwType),
    ("until", TokenKind::KwUntil),
    ("use", TokenKind::KwUse),
    ("when", TokenKind::KwWhen),
    ("while", TokenKind::KwWhile),
    ("with", TokenKind::KwWith),
    ("xor", TokenKind::KwXor),
];

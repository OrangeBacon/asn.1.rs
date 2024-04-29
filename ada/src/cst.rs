/// The description of a single tree node within the CST
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TreeKind {
    ErrorTree,
    File,
    Pragma,
    PragmaArgumentList,
    PragmaArgument,
}

/// The output from the parser that specifies how a CST should be constructed
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Event {
    Open { kind: TreeKind },
    Advance,
    Close,
}

/// Error type that can be shown to a user of this compiler
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Diagnostic {
    name: String,
}

pub type Result<T = (), E = Diagnostic> = std::result::Result<T, E>;

impl Diagnostic {
    /// Construct a new diagnostic
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

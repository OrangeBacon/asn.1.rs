use std::{error::Error, fmt::Display, ops::Range};

use crate::{compiler::SourceId, lexer::LexerError};

/// Any kind of error reported by the compiler
#[derive(Debug)]
pub struct Diagnostic {
    /// Unique error code reference
    pub error_code: String,

    /// Severity of the error
    pub level: Level,

    /// The compiler internal error that triggered this diagnostic
    pub cause: Option<Box<dyn Error + 'static>>,

    /// Name of the diagnostic
    pub name: String,

    /// All labels with information about this diagnostic
    pub labels: Vec<Label>,
}

/// Reference to a source file
#[derive(Debug)]
pub struct Label {
    /// The source file
    pub source: Option<SourceId>,

    /// Location within the source file that the diagnostic should be shown at
    pub location: Option<Range<usize>>,

    /// The message to display to the user.
    pub message: String,
}

/// Severity of a given diagnostic
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    /// A fatal error
    Error,

    /// Should be fixed but the compiler can still continue.
    Warning,

    /// Notice about some code
    Note,
}

/// A result containing a diagnostic, the default error type for the compiler
pub type Result<T = ()> = std::result::Result<T, Diagnostic>;

impl Diagnostic {
    /// Create a new diagnostic
    fn new(level: Level, code: String) -> Self {
        Diagnostic {
            error_code: code,
            level,
            cause: None,
            name: String::new(),
            labels: vec![],
        }
    }

    /// Create an error diagnostic
    pub(crate) fn error(code: impl Into<String>) -> Self {
        Self::new(Level::Error, code.into())
    }

    /// Set the internal error that caused this error
    pub(crate) fn cause(self, value: impl Error + 'static) -> Self {
        Self {
            cause: Some(Box::new(value)),
            ..self
        }
    }

    /// Set the descriptive name of an error
    pub(crate) fn name(self, value: impl Into<String>) -> Self {
        Self {
            name: value.into(),
            ..self
        }
    }

    /// Add a label to the diagnostic
    pub(crate) fn label(mut self, label: impl Into<Label>) -> Self {
        self.labels.push(label.into());
        self
    }
}

impl Label {
    /// Create a new source label
    pub(crate) fn new() -> Label {
        Label {
            source: None,
            location: None,
            message: String::new(),
        }
    }

    /// Set the message for this label
    pub(crate) fn message(self, value: impl Into<String>) -> Self {
        Self {
            message: value.into(),
            ..self
        }
    }

    /// Set the source file for this label
    pub fn source(self, id: SourceId) -> Self {
        Self {
            source: Some(id),
            ..self
        }
    }

    /// Set the location within the source file for this label
    pub fn loc(self, loc: Range<usize>) -> Self {
        Self {
            location: Some(loc),
            ..self
        }
    }
}

impl Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {:04}: {}", self.level, self.error_code, self.name)?;

        for label in &self.labels {
            if let Some(source) = label.source {
                writeln!(f)?;

                write!(f, "\t{:?} [{source:?}", self.level)?;
                if let Some(location) = &label.location {
                    write!(f, "@{}..{}", location.start, location.end)?;
                }
                write!(f, "]: {}", label.message)?;
            }
        }

        for label in &self.labels {
            if label.source.is_none() {
                writeln!(f)?;
                write!(f, "\t{:?}: {}", self.level, label.message)?;
            }
        }

        Ok(())
    }
}

impl Error for Diagnostic {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause.as_deref()
    }
}

impl From<LexerError> for Diagnostic {
    fn from(value: LexerError) -> Self {
        Diagnostic::error("Asn::Lexer::Error").cause(value)
    }
}

impl From<&str> for Label {
    fn from(value: &str) -> Self {
        Label::new().message(value)
    }
}

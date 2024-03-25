use std::{collections::HashMap, error::Error, ops::Range};

use ariadne::{Config, IndexType, ReportKind};
use asn1::{AsnCompiler, Diagnostic, Level, SourceId};

type Report = ariadne::Report<'static, (SourceId, Range<usize>)>;

/// Helper function to convert asn.1 compiler diagnostics to a prettier format.
pub fn to_error(diag: &Diagnostic) -> Result<Report, Box<dyn Error>> {
    let kind = match diag.level {
        Level::Error => ReportKind::Error,
        Level::Warning => ReportKind::Warning,
        Level::Note => ReportKind::Advice,
    };

    let source = diag.labels.first().and_then(|l| l.source);
    let offset = diag
        .labels
        .first()
        .and_then(|l| l.location.clone())
        .map(|l| l.start);
    let (Some(source), Some(offset)) = (source, offset) else {
        return Err("Unable to get source location from diagnostic".into());
    };

    let mut report = Report::build(kind, source, offset)
        .with_code(&diag.error_code)
        .with_message(&diag.name)
        .with_config(Config::default().with_index_type(IndexType::Byte));

    let mut note: Option<String> = None;
    for label in &diag.labels {
        let (Some(source), Some(offset)) = (label.source, &label.location) else {
            note = Some(note.unwrap_or_default() + "\n" + &label.message);
            continue;
        };
        report.add_label(ariadne::Label::new((source, offset.clone())).with_message(&label.message))
    }

    if let Some(note) = note {
        report.set_note(note);
    }

    Ok(report.finish())
}

/// Source file cache provider for the Asn compiler
pub struct AsnCompilerCache<'a> {
    cache: HashMap<SourceId, ariadne::Source<&'a str>>,
    compiler: &'a AsnCompiler,
}

impl<'a> ariadne::Cache<SourceId> for AsnCompilerCache<'a> {
    type Storage = &'a str;

    fn fetch(
        &mut self,
        id: &SourceId,
    ) -> Result<&ariadne::Source<Self::Storage>, Box<dyn std::fmt::Debug + '_>> {
        Ok(self
            .cache
            .entry(*id)
            .or_insert_with(|| ariadne::Source::from(self.compiler.source_text(*id))))
    }

    fn display<'b>(&self, id: &'b SourceId) -> Option<Box<dyn std::fmt::Display + 'b>> {
        Some(Box::new(self.compiler.source_name(*id).to_string()))
    }
}

impl<'a> AsnCompilerCache<'a> {
    pub fn new(compiler: &'a AsnCompiler) -> Self {
        AsnCompilerCache {
            cache: HashMap::new(),
            compiler,
        }
    }
}

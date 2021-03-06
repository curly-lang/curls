use codespan_reporting::diagnostic;
use codespan_reporting::diagnostic::{LabelStyle, Severity, Label};
use codespan_reporting::files;
use codespan_reporting::files::{Files, SimpleFiles};
use tower_lsp::lsp_types::{Diagnostic, Range, Position, DiagnosticSeverity, DiagnosticRelatedInformation, Location, Url, NumberOrString};
use std::convert::TryFrom;

fn from(location: &files::Location) -> Position {
    Position::new(u64::try_from(location.line_number - 1).expect("cannot convert usize to 64"), u64::try_from(location.column_number - 1).expect("cannot convert usize to u64"))
}

fn label_range(label: &Label<usize>, files: &SimpleFiles<&String, String>) -> Option<Range> {
    if let Ok(start) = files.location(label.file_id, label.range.start) {
        if let Ok(end) = files.location(label.file_id, label.range.end) {
            return Some(Range::new(from(&start), from(&end)))
        }
    }
    
    None
}

fn use_label_message(label: &Label<usize>) -> bool {
    if label.message == "Curried function found here" || label.message.is_empty() {
        return false
    }

    true
}

pub fn convert_diagnostics(raw_diagnostics: &Vec<diagnostic::Diagnostic<usize>>, files: &SimpleFiles<&String, String>, diagnostics: &mut Vec<Diagnostic>) {
    for raw_diagnostic in raw_diagnostics {
        if let Some(label) = raw_diagnostic.labels.iter().find(|label| label.style == LabelStyle::Primary) {
            if let Some(range) = label_range(label, files) {
                diagnostics.push(Diagnostic {
                    message: if use_label_message(label) {
                        label.message.clone()
                    } else {
                        raw_diagnostic.message.clone()
                    },
                    code: match &raw_diagnostic.code {
                        Some(code) => Some(NumberOrString::String(code.clone())),
                        None => None
                    },
                    range: range,
                    severity: Some(match raw_diagnostic.severity {
                        Severity::Error => DiagnosticSeverity::Error,
                        Severity::Warning => DiagnosticSeverity::Warning,
                        Severity::Note => DiagnosticSeverity::Information,
                        Severity::Help => DiagnosticSeverity::Hint,
                        Severity::Bug => DiagnosticSeverity::Error
                    }),
                    source: Some(String::from("curls")),
                    related_information: Some(raw_diagnostic.labels.iter().filter_map(|secondary_label| {
                        if secondary_label.style == LabelStyle::Secondary {
                            if let Some(range) = label_range(secondary_label, files) {
                                return Some(DiagnosticRelatedInformation {
                                    message: secondary_label.message.clone(),
                                    location: Location::new(Url::parse(files.get(secondary_label.file_id).expect("unable to get file for file id").name()).expect("unable to parse url"), range)
                                })
                            }
                        }

                        None
                    }).collect()),
                    tags: None,
                })
            }
        }
    }
}
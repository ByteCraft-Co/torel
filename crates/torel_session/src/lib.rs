use std::path::PathBuf;

use torel_diagnostics::{Diagnostic, FileId, Label, Span};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileSession {
    pub input: PathBuf,
    pub emit: EmitKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmitKind {
    Check,
    LlvmIr,
    Object,
    Binary,
}

impl CompileSession {
    #[must_use]
    pub fn check(input: impl Into<PathBuf>) -> Self {
        Self {
            input: input.into(),
            emit: EmitKind::Check,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineCol {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    pub id: FileId,
    pub path: PathBuf,
    pub text: String,
    pub line_starts: Vec<usize>,
}

impl SourceFile {
    #[must_use]
    pub fn new(id: FileId, path: impl Into<PathBuf>, text: String) -> Self {
        let mut line_starts = vec![0];

        for (index, byte) in text.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push(index + 1);
            }
        }

        Self {
            id,
            path: path.into(),
            text,
            line_starts,
        }
    }

    #[must_use]
    pub fn byte_to_line_col(&self, byte: usize) -> LineCol {
        let line_index = match self.line_starts.binary_search(&byte) {
            Ok(index) => index,
            Err(index) => index.saturating_sub(1),
        };
        let line_start = self.line_starts[line_index];

        LineCol {
            line: line_index + 1,
            column: byte.saturating_sub(line_start) + 1,
        }
    }

    #[must_use]
    pub fn line_text(&self, line: usize) -> Option<&str> {
        let line_index = line.checked_sub(1)?;
        let start = *self.line_starts.get(line_index)?;
        let end = self
            .line_starts
            .get(line_index + 1)
            .copied()
            .unwrap_or(self.text.len());

        Some(self.text[start..end].trim_end_matches(['\r', '\n']))
    }
}

#[must_use]
pub fn render_diagnostic(source: &SourceFile, diagnostic: &Diagnostic) -> String {
    let mut output = format!("{}: {}\n", diagnostic.severity.as_str(), diagnostic.message);

    if let Some(label) = diagnostic.labels.first() {
        render_label(source, label, &mut output);
    }

    for note in &diagnostic.notes {
        output.push_str("note: ");
        output.push_str(note);
        output.push('\n');
    }

    output
}

fn render_label(source: &SourceFile, label: &Label, output: &mut String) {
    let Span { start, end } = label.span;
    let location = source.byte_to_line_col(start);
    let path = source.path.display();

    output.push_str(&format!(
        " --> {path}:{}:{}\n",
        location.line, location.column
    ));
    output.push_str("  |\n");

    let Some(line_text) = source.line_text(location.line) else {
        return;
    };

    output.push_str(&format!("{} | {line_text}\n", location.line));
    output.push_str("  | ");

    let underline_start = location.column.saturating_sub(1);
    for _ in 0..underline_start {
        output.push(' ');
    }

    let underline_width = end.saturating_sub(start).max(1);
    for _ in 0..underline_width {
        output.push('^');
    }

    if let Some(message) = &label.message {
        output.push(' ');
        output.push_str(message);
    }

    output.push('\n');
}

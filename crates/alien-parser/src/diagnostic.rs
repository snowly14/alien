//! Source locations and errors produced during parsing and extraction.

use oxc_span::Span;

pub(crate) fn at(message: &str, span: Span) -> String {
    format!("{message} (bytes {}..{})", span.start, span.end)
}

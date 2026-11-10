use std::path::Path;

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_parser::Parser;
use oxc_span::SourceType;

/// Parses a TypeScript module.
///
/// # Errors
///
/// Returns the parser diagnostics if the source contains syntax errors.
pub fn parse<'a>(allocator: &'a Allocator, source: &'a str) -> Result<Program<'a>, Vec<String>> {
    parse_with_source_type(allocator, source, SourceType::ts())
}

pub(crate) fn parse_file<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    path: &Path,
) -> Result<Program<'a>, Vec<String>> {
    let source_type = SourceType::from_path(path).map_err(|error| vec![error.to_string()])?;
    parse_with_source_type(allocator, source, source_type)
}

fn parse_with_source_type<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    source_type: SourceType,
) -> Result<Program<'a>, Vec<String>> {
    let ret = Parser::new(allocator, source, source_type).parse();
    if ret.diagnostics.is_empty() {
        Ok(ret.program)
    } else {
        Err(ret.diagnostics.iter().map(ToString::to_string).collect())
    }
}

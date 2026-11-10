#![doc = env!("CARGO_PKG_DESCRIPTION")]

mod diagnostic;
pub mod extract;
pub mod typescript;

use std::path::Path;

use alien_ir::Style;
use oxc_allocator::Allocator;
use oxc_ast_visit::Visit;

use extract::{Expr, create::CreateFinder};

/// Extracts the object arguments of Alien style calls from a TypeScript module.
///
/// # Errors
///
/// Returns syntax diagnostics or errors in the extracted objects.
pub fn extract_styles(source: &str) -> Result<Vec<Expr>, Vec<String>> {
    let allocator = Allocator::default();
    let program = typescript::parse(&allocator, source)?;
    let mut finder = CreateFinder::default();
    finder.visit_program(&program);
    if finder.errors.is_empty() {
        Ok(finder.styles)
    } else {
        Err(finder.errors)
    }
}

/// Reads top-level Alien styles from a TypeScript or TSX module into the IR.
///
/// The path selects the source format. The caller supplies the file contents.
///
/// # Errors
///
/// Returns diagnostics for invalid source or unsupported Alien style declarations.
pub fn parse_styles(source: &str, path: &Path) -> Result<Vec<Style>, Vec<String>> {
    let allocator = Allocator::default();
    let program = typescript::parse_file(&allocator, source, path)?;
    extract::lower_module(&program)
}

#[cfg(test)]
mod tests;

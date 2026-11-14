#![doc = env!("CARGO_PKG_DESCRIPTION")]

mod diagnostic;
pub mod extract;
pub mod typescript;

use std::{ops::Range, path::Path};

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

/// Styles and source byte offsets for one `alien.create()` call.
#[derive(Debug)]
pub struct StyleBlock {
    pub span: Range<usize>,
    pub styles: Vec<Style>,
}

/// Reads the style calls in a module while retaining their source ranges.
///
/// # Errors
///
/// Returns diagnostics for invalid source or unsupported style declarations.
pub fn parse_style_blocks(source: &str, path: &Path) -> Result<Vec<StyleBlock>, Vec<String>> {
    let allocator = Allocator::default();
    let program = typescript::parse_file(&allocator, source, path)?;
    extract::lower_blocks(&program, alien_ir::Mode::Free)
}

#[cfg(test)]
mod tests;

/// Reads styles using the configured design-system policy.
///
/// # Errors
/// Returns diagnostics for invalid or unsupported source.
pub fn parse_style_blocks_with_mode(
    source: &str,
    path: &Path,
    mode: alien_ir::Mode,
) -> Result<Vec<StyleBlock>, Vec<String>> {
    let allocator = Allocator::default();
    let program = typescript::parse_file(&allocator, source, path)?;
    extract::lower_blocks(&program, mode)
}

/// Statically reads an `alien.globals()` configuration without executing JavaScript.
///
/// # Errors
/// Returns diagnostics for invalid configuration.
pub fn parse_globals(source: &str, path: &Path) -> Result<alien_ir::GlobalConfig, Vec<String>> {
    let allocator = Allocator::default();
    let program = typescript::parse_file(&allocator, source, path)?;
    extract::globals::lower(&program).map_err(|error| vec![error])
}

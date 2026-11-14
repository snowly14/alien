mod border;
mod color;
pub mod create;
mod declaration;
mod expression;
pub(crate) mod globals;
mod imports;
mod keyword;
mod length;
mod motion;
mod object;
mod standard;
pub mod style;
pub mod token;
mod usage;

use alien_ir::Style;
use oxc_ast::ast::{ObjectExpression, Program};
use oxc_span::Span;

use create::{create_object, for_each_module_call, is_create_call};
use imports::Imports;

use crate::StyleBlock;

#[derive(Debug)]
pub enum Expr {
    Number(f64),
    Str(String),
    Object(Vec<(String, Expr)>),
    Path(Vec<PathPart>),
}

#[derive(Debug)]
pub enum PathPart {
    Field(String),
    Index(u32),
}

pub(crate) fn lower_module(program: &Program<'_>) -> Result<Vec<Style>, Vec<String>> {
    let mut styles = Vec::new();
    for_each_style_block(program, alien_ir::Mode::Free, |_, object, imports| {
        style::lower_styles(object, imports, &mut styles)
    })?;
    Ok(styles)
}

pub(crate) fn lower_blocks(
    program: &Program<'_>,
    mode: alien_ir::Mode,
) -> Result<Vec<StyleBlock>, Vec<String>> {
    let mut blocks = Vec::new();
    for_each_style_block(program, mode, |span, object, imports| {
        let mut styles = Vec::new();
        style::lower_styles(object, imports, &mut styles)?;
        blocks.push(StyleBlock {
            span: span.start as usize..span.end as usize,
            styles,
        });
        Ok(())
    })?;
    Ok(blocks)
}

fn for_each_style_block(
    program: &Program<'_>,
    mode: alien_ir::Mode,
    mut lower: impl FnMut(Span, &ObjectExpression<'_>, &Imports<'_>) -> Result<(), String>,
) -> Result<(), Vec<String>> {
    let mut imports = Imports::from_program(program);
    imports.mode = mode;
    usage::validate_usage(program, &imports)?;
    if !imports.has_alien() {
        return Ok(());
    }

    let mut errors = Vec::new();
    for_each_module_call(program, |call| {
        if is_create_call(call, |name| imports.is_alien(name)) {
            let result = create_object(call).and_then(|object| lower(call.span, object, &imports));
            if let Err(error) = result {
                errors.push(error);
            }
        }
    });

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

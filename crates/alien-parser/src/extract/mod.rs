pub mod create;
mod expression;
mod imports;
mod object;
pub mod style;
pub mod token;

use alien_ir::Style;
use oxc_ast::ast::Program;

use create::{create_object, for_each_module_call, is_create_call};
use imports::Imports;

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
    let imports = Imports::from_program(program);
    if !imports.has_alien() {
        return Ok(Vec::new());
    }

    let mut styles = Vec::new();
    let mut errors = Vec::new();
    for_each_module_call(program, |call| {
        if is_create_call(call, |name| imports.is_alien(name)) {
            let result = create_object(call)
                .and_then(|object| style::lower_styles(object, &imports, &mut styles));
            if let Err(error) = result {
                errors.push(error);
            }
        }
    });

    if errors.is_empty() {
        Ok(styles)
    } else {
        Err(errors)
    }
}

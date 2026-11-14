use oxc_ast::ast::{
    CallExpression, Declaration, Expression, ImportDeclaration, ObjectExpression, Program,
    Statement,
};
use oxc_ast_visit::Visit;

use super::{
    Expr,
    expression::convert_object,
    imports::{ImportKind, for_each_alien_binding, for_each_import},
};
use crate::diagnostic::at;

/// Collects raw arguments to Alien's style calls.
#[derive(Default)]
pub struct CreateFinder {
    alien_names: Vec<String>,
    pub styles: Vec<Expr>,
    pub errors: Vec<String>,
}

impl<'source> Visit<'source> for CreateFinder {
    fn visit_program(&mut self, program: &Program<'source>) {
        for_each_import(program, |import| self.visit_import_declaration(import));
        if !self.alien_names.is_empty() {
            for_each_module_call(program, |call| self.visit_call_expression(call));
        }
    }

    fn visit_import_declaration(&mut self, import: &ImportDeclaration<'source>) {
        for_each_alien_binding(import, |kind, name| {
            if matches!(kind, ImportKind::Alien) {
                self.alien_names.push(name.to_owned());
            }
        });
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'source>) {
        if is_create_call(call, |name| {
            self.alien_names.iter().any(|binding| binding == name)
        }) {
            match create_object(call).and_then(convert_object) {
                Ok(expr) => self.styles.push(expr),
                Err(error) => self.errors.push(error),
            }
        }
    }
}

pub(super) fn for_each_module_call<'source>(
    program: &Program<'source>,
    mut visit: impl FnMut(&CallExpression<'source>),
) {
    // Direct initializers in the module share the scope of its imports.
    for statement in &program.body {
        if let Statement::ExportDefaultDeclaration(export) = statement
            && let Some(Expression::CallExpression(call)) = export
                .declaration
                .as_expression()
                .map(Expression::get_inner_expression)
        {
            visit(call);
            continue;
        }
        let declaration = match statement {
            Statement::VariableDeclaration(declaration) => declaration,
            Statement::ExportDeclaration(export) => {
                let Declaration::VariableDeclaration(declaration) = &export.declaration else {
                    continue;
                };
                declaration
            }
            _ => continue,
        };

        for variable in &declaration.declarations {
            if let Some(Expression::CallExpression(call)) =
                variable.init.as_ref().map(Expression::get_inner_expression)
            {
                visit(call);
            }
        }
    }
}

pub(super) fn is_create_call(
    call: &CallExpression<'_>,
    is_alien: impl FnOnce(&str) -> bool,
) -> bool {
    let Expression::StaticMemberExpression(member) = call.callee.get_inner_expression() else {
        return false;
    };
    if member.property.name.as_str() != "create" {
        return false;
    }
    let Expression::Identifier(receiver) = member.object.get_inner_expression() else {
        return false;
    };
    is_alien(receiver.name.as_str())
}

pub(super) fn create_object<'node, 'source>(
    call: &'node CallExpression<'source>,
) -> Result<&'node ObjectExpression<'source>, String> {
    let [argument] = call.arguments.as_slice() else {
        return Err(at("alien.create() expects exactly one object", call.span));
    };
    let Some(Expression::ObjectExpression(object)) = argument
        .as_expression()
        .map(Expression::get_inner_expression)
    else {
        return Err(at("alien.create() expects an object", call.span));
    };

    Ok(object)
}

//! Rejects Alien references that would otherwise survive compilation.

use oxc_ast::ast::{
    CallExpression, Expression, IdentifierReference, ImportDeclarationSpecifier, Program,
};
use oxc_ast_visit::{Visit, walk};
use oxc_semantic::{Scoping, SemanticBuilder};
use oxc_span::Span;

use super::{
    create::{for_each_module_call, is_create_call},
    imports::{Imports, for_each_import},
};
use crate::diagnostic::at;

pub(super) fn validate_usage(
    program: &Program<'_>,
    imports: &Imports<'_>,
) -> Result<(), Vec<String>> {
    let mut roots = Vec::new();
    for_each_import(program, |import| {
        if import.source.value.as_str() != Some("alien") || import.import_kind.is_type() {
            return;
        }
        for specifier in import.specifiers.iter().flatten() {
            match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(specifier)
                    if !specifier.import_kind.is_type()
                        && specifier.imported.name().as_str() == "alien" =>
                {
                    roots.push(specifier.local.name.as_str());
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                    roots.push(specifier.local.name.as_str());
                }
                _ => {}
            }
        }
    });
    if roots.is_empty() {
        return Ok(());
    }
    let mut allowed = Vec::new();
    for_each_module_call(program, |call| {
        if is_create_call(call, |name| imports.is_alien(name)) {
            allowed.push(call.span);
        }
    });
    let semantic = SemanticBuilder::new()
        .with_check_syntax_error(true)
        .build(program);
    if !semantic.diagnostics.is_empty() {
        return Err(semantic
            .diagnostics
            .iter()
            .map(ToString::to_string)
            .collect());
    }
    let mut visitor = Usage {
        roots,
        allowed,
        scoping: semantic.semantic.scoping(),
        errors: Vec::new(),
    };
    visitor.visit_program(program);
    if visitor.errors.is_empty() {
        Ok(())
    } else {
        Err(visitor.errors)
    }
}

struct Usage<'a> {
    roots: Vec<&'a str>,
    allowed: Vec<Span>,
    scoping: &'a Scoping,
    errors: Vec<String>,
}

impl<'a> Visit<'a> for Usage<'_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if !self.allowed.contains(&call.span) {
            walk::walk_call_expression(self, call);
        }
    }

    fn visit_expression(&mut self, expression: &Expression<'a>) {
        // Reading the built-in palette does not invoke the compile-only API.
        if let Expression::StaticMemberExpression(member) = expression
            && member.property.name.as_str() == "color"
            && matches!(
                member.object.get_inner_expression(),
                Expression::Identifier(_)
            )
        {
            return;
        }
        walk::walk_expression(self, expression);
    }

    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        if !self.roots.contains(&identifier.name.as_str()) {
            return;
        }
        let Some(symbol) = identifier
            .reference_id
            .get()
            .and_then(|id| self.scoping.get_reference(id).symbol_id())
        else {
            return;
        };
        if self.scoping.symbol_scope_id(symbol) == self.scoping.root_scope_id() {
            self.errors.push(at("Alien calls must use a named import and a direct top-level variable initializer or default export; dynamic calls and aliases are not supported", identifier.span));
        }
    }
}

use oxc_ast::ast::{Expression, ObjectExpression};
use oxc_span::GetSpan;

use super::{Expr, object::read_property, token::collect_path};
use crate::diagnostic::at;

pub(super) fn convert_object(object: &ObjectExpression<'_>) -> Result<Expr, String> {
    let mut properties = Vec::with_capacity(object.properties.len());
    for property in &object.properties {
        let (key, value) = read_property(property)?;
        properties.push((key.into_owned(), convert_expr(value)?));
    }
    Ok(Expr::Object(properties))
}

fn convert_expr(expr: &Expression<'_>) -> Result<Expr, String> {
    match expr.get_inner_expression() {
        Expression::NumericLiteral(number) => Ok(Expr::Number(number.value)),
        Expression::StringLiteral(string) => string
            .value
            .as_str()
            .map(|value| Expr::Str(value.to_owned()))
            .ok_or_else(|| at("expected a UTF-8 string", string.span)),
        Expression::ObjectExpression(object) => convert_object(object),
        Expression::Identifier(_)
        | Expression::StaticMemberExpression(_)
        | Expression::ComputedMemberExpression(_) => {
            let mut parts = Vec::new();
            collect_path(expr, &mut parts)?;
            Ok(Expr::Path(parts))
        }
        _ => Err(at("unsupported style value", expr.span())),
    }
}

use std::borrow::Cow;

use oxc_ast::{
    StaticPropertyName,
    ast::{Expression, ObjectExpression, ObjectPropertyKind, PropertyKind},
};
use oxc_span::GetSpan;

use crate::diagnostic::at;

pub(super) fn read_property<'node, 'source>(
    property: &'node ObjectPropertyKind<'source>,
) -> Result<(Cow<'source, str>, &'node Expression<'source>), String> {
    let property = match property {
        ObjectPropertyKind::ObjectProperty(property) => property,
        ObjectPropertyKind::SpreadProperty(spread) => {
            return Err(at("spread properties are not supported", spread.span));
        }
    };

    if property.computed || property.method || property.kind != PropertyKind::Init {
        return Err(at(
            "expected a static property key and value",
            property.span,
        ));
    }

    let name = property
        .key
        .static_name()
        .and_then(StaticPropertyName::into_cow_str)
        .ok_or_else(|| at("expected a static UTF-8 property key", property.span))?;

    Ok((name, &property.value))
}

pub(super) fn expect_object<'node, 'source>(
    expr: &'node Expression<'source>,
    context: &str,
) -> Result<&'node ObjectExpression<'source>, String> {
    match expr.get_inner_expression() {
        Expression::ObjectExpression(object) => Ok(object),
        _ => Err(at(&format!("`{context}` expects an object"), expr.span())),
    }
}

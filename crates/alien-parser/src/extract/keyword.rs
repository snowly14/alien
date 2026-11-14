use oxc_ast::ast::Expression;
use oxc_span::GetSpan;

use super::imports::Imports;
use crate::diagnostic::at;

pub(super) fn css_keyword<'source>(
    value: &Expression<'source>,
    _family: &str,
    imports: &Imports<'_>,
) -> Option<&'source str> {
    match value.get_inner_expression() {
        Expression::StringLiteral(string) => string.value.as_str(),
        Expression::StaticMemberExpression(member) => {
            let Expression::StaticMemberExpression(category) = member.object.get_inner_expression()
            else {
                return None;
            };
            let Expression::Identifier(root) = category.object.get_inner_expression() else {
                return None;
            };
            imports
                .is_css(root.name.as_str())
                .then(|| member.property.name.as_str())
        }
        _ => None,
    }
}

pub(super) fn keyword(
    value: &Expression<'_>,
    family: &str,
    imports: &Imports<'_>,
    accepted: &[(&'static str, &'static str)],
) -> Result<&'static str, String> {
    css_keyword(value, family, imports)
        .and_then(|name| {
            accepted
                .iter()
                .find_map(|&(key, css)| (key == name || css == name).then_some(css))
        })
        .ok_or_else(|| at(&format!("expected a `{family}` keyword"), value.span()))
}

/// Recognizes the declared CSS spelling or its camelCase member name.
pub(super) fn standard_keyword(
    value: &Expression<'_>,
    family: &str,
    imports: &Imports<'_>,
    accepted: &[&[&'static str]],
) -> Result<&'static str, String> {
    let is_member = matches!(
        value.get_inner_expression(),
        Expression::StaticMemberExpression(_)
    );
    css_keyword(value, family, imports)
        .and_then(|name| {
            accepted
                .iter()
                .flat_map(|set| set.iter().copied())
                .find(|css| *css == name || (is_member && matches_member(name, css)))
        })
        .ok_or_else(|| at(&format!("expected a `{family}` keyword"), value.span()))
}

fn matches_member(member: &str, css: &str) -> bool {
    let mut member = member.bytes();
    let mut uppercase = false;
    for byte in css.bytes() {
        if matches!(byte, b'-' | b' ') {
            uppercase = true;
            continue;
        }
        let expected = if uppercase {
            byte.to_ascii_uppercase()
        } else {
            byte
        };
        if member.next() != Some(expected) {
            return false;
        }
        uppercase = false;
    }
    member.next().is_none()
}

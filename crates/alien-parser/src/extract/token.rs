use alien_ir::{ColorToken, TokenFamily, Value};
use oxc_ast::ast::Expression;
use oxc_span::GetSpan;

use super::{PathPart, imports::Imports};
use crate::diagnostic::at;

pub(super) fn collect_path(expr: &Expression<'_>, parts: &mut Vec<PathPart>) -> Result<(), String> {
    match expr.get_inner_expression() {
        Expression::Identifier(identifier) => {
            parts.push(PathPart::Field(identifier.name.to_string()));
        }
        Expression::StaticMemberExpression(member) => {
            collect_path(&member.object, parts)?;
            parts.push(PathPart::Field(member.property.name.to_string()));
        }
        Expression::ComputedMemberExpression(member) => {
            collect_path(&member.object, parts)?;
            parts.push(PathPart::Index(token_index(&member.expression)?));
        }
        _ => return Err(at("invalid token reference", expr.span())),
    }
    Ok(())
}

pub(super) fn lower_color_token(
    expr: &Expression<'_>,
    imports: &Imports<'_>,
    property: &str,
) -> Result<ColorToken, String> {
    let invalid = || {
        at(
            &format!("{property} expects a color token imported from `alien`"),
            expr.span(),
        )
    };
    let (member, index) = match expr.get_inner_expression() {
        Expression::StaticMemberExpression(member) => (member, None),
        Expression::ComputedMemberExpression(computed) => {
            let Expression::StaticMemberExpression(member) = computed.object.get_inner_expression()
            else {
                return Err(invalid());
            };
            (member, Some(&computed.expression))
        }
        _ => return Err(invalid()),
    };

    let Expression::StaticMemberExpression(category) = member.object.get_inner_expression() else {
        return Err(invalid());
    };
    let Expression::Identifier(root) = category.object.get_inner_expression() else {
        return Err(invalid());
    };
    if category.property.name.as_str() != "color" || !imports.is_color_root(root.name.as_str()) {
        return Err(invalid());
    }

    let name = member.property.name.as_str();
    match index {
        Some(index) => {
            let step = u16::try_from(token_index(index)?)
                .map_err(|_| at("color shade index is too large", index.span()))?;
            Ok(ColorToken::Shade {
                palette: name.to_owned(),
                step,
            })
        }
        None => match name {
            "black" => Ok(ColorToken::Black),
            "white" => Ok(ColorToken::White),
            name => Ok(ColorToken::Named(name.to_owned())),
        },
    }
}

pub(super) fn lower_token(
    value: &Expression<'_>,
    imports: &Imports<'_>,
    family: TokenFamily,
) -> Result<Value, String> {
    let expected = family.as_str();
    if let Some((category, name)) = named_token_path(value, imports)
        && category == expected
    {
        return Ok(Value::Token(family, name.to_owned()));
    }
    Err(at(
        &format!("expected a `{expected}` token imported from `alien`"),
        value.span(),
    ))
}

pub(super) fn named_token_path<'source>(
    expr: &Expression<'source>,
    imports: &Imports<'_>,
) -> Option<(&'source str, &'source str)> {
    let Expression::StaticMemberExpression(member) = expr.get_inner_expression() else {
        return None;
    };
    let Expression::StaticMemberExpression(category) = member.object.get_inner_expression() else {
        return None;
    };
    let Expression::Identifier(root) = category.object.get_inner_expression() else {
        return None;
    };
    imports.is_token_root(root.name.as_str()).then(|| {
        (
            category.property.name.as_str(),
            member.property.name.as_str(),
        )
    })
}

fn token_index(expr: &Expression<'_>) -> Result<u32, String> {
    let Expression::NumericLiteral(number) = expr.get_inner_expression() else {
        return Err(at("expected a numeric token index", expr.span()));
    };
    if !(0.0..=f64::from(u32::MAX)).contains(&number.value) || number.value.fract() != 0.0 {
        return Err(at(
            "token index must be a non-negative integer",
            number.span,
        ));
    }
    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let index = number.value as u32;
    Ok(index)
}

use alien_ir::{Length, LengthUnit, Value};
use oxc_ast::ast::Expression;
use oxc_span::GetSpan;

use super::imports::Imports;
use crate::diagnostic::at;

pub(super) fn lower_length(expr: &Expression<'_>, imports: &Imports<'_>) -> Result<Length, String> {
    lower_length_at_depth(expr, imports, 0)
}

fn lower_length_at_depth(
    expr: &Expression<'_>,
    imports: &Imports<'_>,
    depth: usize,
) -> Result<Length, String> {
    if depth > 64 {
        return Err(at(
            "length expressions cannot exceed 64 levels",
            expr.span(),
        ));
    }
    if let Expression::StringLiteral(string) = expr.get_inner_expression() {
        if let Some(text) = string.value.as_str() {
            for unit in ["rem", "svh", "dvh", "px", "em", "ch", "vw", "vh"] {
                if let Some(number) = text
                    .strip_suffix(unit)
                    .and_then(|number| number.parse::<f64>().ok())
                    .filter(|number| number.is_finite())
                {
                    return Ok(Length::Literal {
                        value: number,
                        unit: LengthUnit::from_name(unit).expect("known unit"),
                    });
                }
            }
        }
        return Err(at(
            "expected a supported CSS length such as 12px or 1.5rem",
            expr.span(),
        ));
    }
    let invalid = || {
        at(
            "expected a length helper imported from `alien/values`",
            expr.span(),
        )
    };
    let Expression::CallExpression(call) = expr.get_inner_expression() else {
        return Err(invalid());
    };
    match call.callee.get_inner_expression() {
        Expression::Identifier(binding) => {
            let unit = imports
                .value_export(binding.name.as_str())
                .and_then(LengthUnit::from_name)
                .ok_or_else(invalid)?;
            let [argument] = call.arguments.as_slice() else {
                return Err(at("a length helper expects one number", call.span));
            };
            let value = argument
                .as_expression()
                .and_then(static_number)
                .filter(|value| value.is_finite())
                .ok_or_else(|| at("expected a finite numeric literal", argument.span()))?;
            Ok(Length::Literal { value, unit })
        }
        Expression::StaticMemberExpression(member)
            if member.property.name.as_str() == "sub"
                && matches!(
                    member.object.get_inner_expression(),
                    Expression::Identifier(binding)
                        if imports.value_export(binding.name.as_str()) == Some("calc")
                ) =>
        {
            let [left, right] = call.arguments.as_slice() else {
                return Err(at("calc.sub() expects two lengths", call.span));
            };
            let (Some(left), Some(right)) = (left.as_expression(), right.as_expression()) else {
                return Err(at("calc.sub() expects two lengths", call.span));
            };
            Ok(Length::Sub {
                left: Box::new(lower_length_at_depth(left, imports, depth + 1)?),
                right: Box::new(lower_length_at_depth(right, imports, depth + 1)?),
            })
        }
        _ => Err(invalid()),
    }
}

pub(super) fn static_number(expr: &Expression<'_>) -> Option<f64> {
    evaluate_number(expr, 0).filter(|number| number.is_finite())
}

fn evaluate_number(expr: &Expression<'_>, depth: usize) -> Option<f64> {
    if depth > 64 {
        return None;
    }
    match expr.get_inner_expression() {
        Expression::NumericLiteral(number) => Some(number.value),
        Expression::UnaryExpression(unary) => {
            let number = evaluate_number(&unary.argument, depth + 1)?;
            match unary.operator.as_str() {
                "-" => Some(-number),
                "+" => Some(number),
                _ => None,
            }
        }
        Expression::BinaryExpression(binary) => {
            let left = evaluate_number(&binary.left, depth + 1)?;
            let right = evaluate_number(&binary.right, depth + 1)?;
            match binary.operator.as_str() {
                "+" => Some(left + right),
                "-" => Some(left - right),
                "*" => Some(left * right),
                "/" => Some(left / right),
                "%" => Some(left % right),
                "**" => Some(left.powf(right)),
                _ => None,
            }
        }
        _ => None,
    }
}

pub(super) fn numeric_helper(
    value: &Expression<'_>,
    imports: &Imports<'_>,
    helper: &str,
) -> Result<Option<f64>, String> {
    helper_argument(value, |binding| {
        imports.value_export(binding) == Some(helper)
    })?
    .map(|argument| {
        static_number(argument)
            .filter(|value| value.is_finite())
            .ok_or_else(|| at("expected a finite numeric literal", argument.span()))
    })
    .transpose()
}

pub(super) fn custom_argument<'node, 'source>(
    value: &'node Expression<'source>,
    imports: &Imports<'_>,
) -> Result<Option<&'node Expression<'source>>, String> {
    helper_argument(value, |binding| imports.is_custom(binding))
}

pub(super) fn helper_argument<'node, 'source>(
    value: &'node Expression<'source>,
    is_helper: impl FnOnce(&str) -> bool,
) -> Result<Option<&'node Expression<'source>>, String> {
    let Expression::CallExpression(call) = value.get_inner_expression() else {
        return Ok(None);
    };
    let Expression::Identifier(binding) = call.callee.get_inner_expression() else {
        return Ok(None);
    };
    if !is_helper(binding.name.as_str()) {
        return Ok(None);
    }
    if let [argument] = call.arguments.as_slice()
        && let Some(argument) = argument.as_expression()
    {
        return Ok(Some(argument));
    }
    Err(at("expected exactly one argument", call.span))
}

pub(super) fn lower_dimension(
    value: &Expression<'_>,
    imports: &Imports<'_>,
    allow_negative: bool,
) -> Result<Value, String> {
    let literal_percent = if let Expression::StringLiteral(string) = value.get_inner_expression() {
        string
            .value
            .as_str()
            .and_then(|text| text.strip_suffix('%'))
            .and_then(|number| number.parse::<f64>().ok())
            .filter(|number| number.is_finite())
    } else {
        None
    };
    let dimension = match literal_percent.or(numeric_helper(value, imports, "percent")?) {
        Some(percent) => Value::Percent(percent),
        None => Value::Length(lower_length(value, imports)?),
    };
    if !allow_negative
        && matches!(dimension, Value::Percent(number) | Value::Length(Length::Literal { value: number, .. }) if number < 0.0)
    {
        return Err(at("expected a non-negative dimension", value.span()));
    }
    Ok(dimension)
}

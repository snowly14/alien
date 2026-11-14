use alien_ir::{GlobalConfig, Mode, Property, Reset, Value};
use oxc_ast::ast::{Expression, Program, Statement};
use oxc_span::GetSpan;

use super::{
    declaration::lower_value,
    imports::Imports,
    length::custom_argument,
    object::{expect_object, read_property},
};
use crate::diagnostic::at;

pub(crate) fn lower(program: &Program<'_>) -> Result<GlobalConfig, String> {
    let imports = Imports::from_program(program);
    let mut config = None;
    for statement in &program.body {
        let expression = match statement {
            Statement::ImportDeclaration(_) | Statement::EmptyStatement(_) => continue,
            Statement::ExportDefaultDeclaration(export) => export.declaration.as_expression(),
            Statement::ExpressionStatement(statement) => Some(&statement.expression),
            _ => None,
        }.ok_or_else(|| at("configuration supports imports and one alien.globals({...}) call; executable statements are not supported", statement.span()))?;
        let Expression::CallExpression(call) = expression.get_inner_expression() else {
            return Err(at("expected alien.globals({...})", expression.span()));
        };
        let Expression::StaticMemberExpression(member) = call.callee.get_inner_expression() else {
            return Err(at("expected alien.globals({...})", call.span));
        };
        if member.property.name.as_str() != "globals"
            || !matches!(member.object.get_inner_expression(), Expression::Identifier(identifier) if imports.is_alien(identifier.name.as_str()))
        {
            return Err(at(
                "expected alien.globals() imported from alien",
                call.span,
            ));
        }
        if config.is_some() {
            return Err(at(
                "only one alien.globals() declaration is allowed",
                call.span,
            ));
        }
        let [argument] = call.arguments.as_slice() else {
            return Err(at("alien.globals() expects one object", call.span));
        };
        let argument = argument
            .as_expression()
            .ok_or_else(|| at("expected a configuration object", argument.span()))?;
        config = Some(lower_config(argument, &imports)?);
    }
    config.ok_or_else(|| "expected one alien.globals() declaration".to_owned())
}

fn lower_config(argument: &Expression<'_>, imports: &Imports<'_>) -> Result<GlobalConfig, String> {
    let mut result = GlobalConfig::default();
    let mut fields = std::collections::BTreeSet::new();
    for property in &expect_object(argument, "globals")?.properties {
        let (family, value) = read_property(property)?;
        if !fields.insert(family.to_string()) {
            return Err(at(
                &format!("duplicate configuration field `{family}`"),
                value.span(),
            ));
        }
        if family == "mode" {
            result.mode = match value.get_inner_expression() {
                Expression::StringLiteral(string) if string.value.as_str() == Some("free") => {
                    Mode::Free
                }
                Expression::StringLiteral(string) if string.value.as_str() == Some("strict") => {
                    Mode::Strict
                }
                _ => return Err(at("mode must be 'free' or 'strict'", value.span())),
            };
            continue;
        }
        if family == "reset" {
            result.reset = parse_reset(value)?;
            continue;
        }
        let target = match family.as_ref() {
            "color" => Property::Color,
            "space" => Property::Padding,
            "radius" => Property::BorderRadius,
            "fontWeight" => Property::FontWeight,
            "fontSize" => Property::FontSize,
            "lineHeight" => Property::LineHeight,
            "duration" => Property::TransitionDuration,
            _ => {
                return Err(at(
                    &format!("unsupported token family `{family}`"),
                    value.span(),
                ));
            }
        };
        for property in &expect_object(value, &family)?.properties {
            let (name, value) = read_property(property)?;
            if name.is_empty()
                || !name.bytes().enumerate().all(|(index, byte)| {
                    byte.is_ascii_alphabetic()
                        || byte == b'_'
                        || (index > 0 && byte.is_ascii_digit())
                })
                || matches!(name.as_ref(), "__proto__" | "constructor" | "prototype")
            {
                return Err(at(
                    "token names must be safe JavaScript identifiers",
                    value.span(),
                ));
            }
            if custom_argument(value, imports)?.is_some() {
                return Err(at(
                    "globals already defines custom values; remove custom()",
                    value.span(),
                ));
            }
            let token = lower_value(target, value, imports)?;
            if matches!(token, Value::Token(_, _))
                || matches!(
                    &token,
                    Value::Color(alien_ir::ColorValue::Token(alien_ir::ColorToken::Named(_)))
                )
            {
                return Err(at(
                    "global tokens must contain literal values, not token aliases",
                    value.span(),
                ));
            }
            let path = format!("{family}.{name}");
            if result.tokens.insert(path.clone(), token).is_some() {
                return Err(at(&format!("duplicate token `{path}`"), value.span()));
            }
        }
    }
    Ok(result)
}

fn parse_reset(value: &Expression<'_>) -> Result<Reset, String> {
    let level = match value.get_inner_expression() {
        Expression::BooleanLiteral(literal) if !literal.value => Some(Reset::Disabled),
        Expression::StringLiteral(literal) => match literal.value.as_str() {
            Some("light") => Some(Reset::Light),
            Some("medium") => Some(Reset::Medium),
            Some("strong") => Some(Reset::Strong),
            _ => None,
        },
        _ => None,
    };
    level.ok_or_else(|| {
        at(
            "reset must be false, 'light', 'medium' or 'strong'",
            value.span(),
        )
    })
}

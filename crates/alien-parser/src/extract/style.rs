use alien_ir::{Condition, Declaration, Property, PseudoClass, Rule, Style};
use oxc_ast::ast::{Expression, ObjectExpression};
use oxc_span::GetSpan;

use super::{
    declaration::lower_value,
    imports::Imports,
    length::helper_argument,
    object::{expect_object, read_property},
};
use crate::diagnostic::at;

pub(super) fn lower_styles(
    object: &ObjectExpression<'_>,
    imports: &Imports<'_>,
    styles: &mut Vec<Style>,
) -> Result<(), String> {
    styles.reserve(object.properties.len());
    for property in &object.properties {
        let (name, value) = read_property(property)?;
        let object = expect_object(value, &name)?;
        let mut rules = Vec::new();
        lower_rules(object, &Condition::default(), imports, &mut rules)
            .map_err(|error| format!("style `{name}`: {error}"))?;
        styles.push(Style {
            name: name.into_owned(),
            rules,
        });
    }
    Ok(())
}

fn lower_rules(
    object: &ObjectExpression<'_>,
    condition: &Condition,
    imports: &Imports<'_>,
    rules: &mut Vec<Rule>,
) -> Result<(), String> {
    for property in &object.properties {
        let (key, value) = read_property(property)?;
        match key.as_ref() {
            "on" => lower_conditions(value, condition, imports, rules)?,
            "border" | "borderTop" | "borderRight" | "borderBottom" | "borderLeft"
            | "transition" | "animation" => {
                let (value, group_important) = unwrap_important(value, imports)?;
                for property in &expect_object(value, &key)?.properties {
                    let (field, value) = read_property(property)?;
                    let property = Property::from_name(&key, &field).ok_or_else(|| {
                        at(&format!("unsupported {key} field `{field}`"), value.span())
                    })?;
                    lower_rule(rules, condition, property, value, imports, group_important)?;
                }
            }
            _ => {
                let property = Property::from_name("", &key).ok_or_else(|| {
                    at(&format!("unsupported style property `{key}`"), value.span())
                })?;
                lower_rule(rules, condition, property, value, imports, false)?;
            }
        }
    }
    Ok(())
}

fn lower_conditions(
    value: &Expression<'_>,
    condition: &Condition,
    imports: &Imports<'_>,
    rules: &mut Vec<Rule>,
) -> Result<(), String> {
    for property in &expect_object(value, "on")?.properties {
        let (key, value) = read_property(property)?;
        if key == "breakpoint" {
            if condition.breakpoint.is_some() {
                return Err(at(
                    "cannot nest a breakpoint inside another breakpoint",
                    value.span(),
                ));
            }
            for property in &expect_object(value, "on.breakpoint")?.properties {
                let (name, value) = read_property(property)?;
                if !alien_tokens::BREAKPOINTS
                    .iter()
                    .any(|(key, _)| *key == name.as_ref())
                {
                    return Err(at(
                        &format!("unknown token: breakpoint.{name}"),
                        value.span(),
                    ));
                }
                let next = Condition {
                    breakpoint: Some(name.into_owned()),
                    pseudo_classes: condition.pseudo_classes,
                };
                lower_rules(
                    expect_object(value, "breakpoint style")?,
                    &next,
                    imports,
                    rules,
                )?;
            }
            continue;
        }

        let pseudo_class = PseudoClass::from_name(&key)
            .ok_or_else(|| at(&format!("unsupported condition `on.{key}`"), value.span()))?;
        let mut next = condition.clone();
        if !next.pseudo_classes.insert(pseudo_class) {
            return Err(at(
                &format!("cannot repeat `on.{key}` inside a condition"),
                value.span(),
            ));
        }
        lower_rules(expect_object(value, &key)?, &next, imports, rules)?;
    }
    Ok(())
}

fn lower_rule(
    rules: &mut Vec<Rule>,
    condition: &Condition,
    property: Property,
    value: &Expression<'_>,
    imports: &Imports<'_>,
    group_important: bool,
) -> Result<(), String> {
    let (inner, important) = unwrap_important(value, imports)?;
    let declaration = Declaration {
        property,
        value: lower_value(property, inner, imports)?,
        important: group_important || important,
    };
    if let Some(existing) = rules.iter().find(|rule| {
        &rule.condition == condition
            && rule
                .declaration
                .property
                .conflicts_with(declaration.property)
    }) {
        let kind = if existing.declaration.property == declaration.property {
            "duplicate"
        } else {
            "overlapping"
        };
        return Err(at(
            &format!(
                "{kind} {} declaration for {condition:?}",
                property.source_name()
            ),
            value.span(),
        ));
    }
    rules.push(Rule {
        condition: condition.clone(),
        declaration,
    });
    Ok(())
}

fn unwrap_important<'node, 'source>(
    value: &'node Expression<'source>,
    imports: &Imports<'_>,
) -> Result<(&'node Expression<'source>, bool), String> {
    let inner = helper_argument(value, |binding| {
        imports.value_export(binding) == Some("important")
    })?;
    Ok((inner.unwrap_or(value), inner.is_some()))
}

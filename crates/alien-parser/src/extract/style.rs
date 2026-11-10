use alien_ir::{Condition, Declaration, Rule, Style};
use oxc_ast::ast::ObjectExpression;
use oxc_span::GetSpan;

use super::{
    imports::Imports,
    object::{expect_object, read_property},
    token::lower_color_token,
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
        lower_rules(object, Condition::Base, imports, &mut rules)
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
    condition: Condition,
    imports: &Imports<'_>,
    rules: &mut Vec<Rule>,
) -> Result<(), String> {
    for property in &object.properties {
        let (key, value) = read_property(property)?;
        match key.as_ref() {
            "backgroundColor" => {
                if rules.iter().any(|rule| rule.condition == condition) {
                    return Err(at(
                        &format!("duplicate backgroundColor declaration for {condition:?}"),
                        value.span(),
                    ));
                }
                let color = lower_color_token(value, imports)?;
                rules.push(Rule {
                    condition,
                    declaration: Declaration::BackgroundColor(color),
                });
            }
            "on" if condition == Condition::Base => {
                let conditions = expect_object(value, "on")?;
                for property in &conditions.properties {
                    let (condition, style) = read_property(property)?;
                    if condition != "hover" {
                        return Err(at(
                            &format!("unsupported condition `on.{condition}`"),
                            style.span(),
                        ));
                    }
                    lower_rules(
                        expect_object(style, "on.hover")?,
                        Condition::Hover,
                        imports,
                        rules,
                    )?;
                }
            }
            _ => {
                return Err(at(
                    &format!("unsupported style property `{key}`"),
                    value.span(),
                ));
            }
        }
    }
    Ok(())
}

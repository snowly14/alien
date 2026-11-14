use alien_ir::{ListSeparator, Property as P, TokenFamily as T, Value};
use oxc_ast::ast::Expression;
use oxc_span::GetSpan;

use super::{
    imports::Imports,
    keyword::{css_keyword, keyword},
    length::{custom_argument, lower_dimension, lower_length, numeric_helper, static_number},
    object::{expect_object, lower_list, read_property},
    token::{lower_token, named_token_path},
};
use crate::diagnostic::at;

const TRANSITION_PROPERTIES: &[(&str, &str)] = &[
    ("all", "all"),
    ("none", "none"),
    ("backgroundColor", "background-color"),
    ("borderColor", "border-color"),
    ("borderRadius", "border-radius"),
    ("boxShadow", "box-shadow"),
    ("color", "color"),
    ("filter", "filter"),
    ("height", "height"),
    ("maxHeight", "max-height"),
    ("maxWidth", "max-width"),
    ("opacity", "opacity"),
    ("outlineColor", "outline-color"),
    ("transform", "transform"),
    ("translate", "translate"),
    ("width", "width"),
];

pub(super) fn lower_value(
    property: P,
    value: &Expression<'_>,
    imports: &Imports<'_>,
) -> Result<Value, String> {
    let (family, accepted): (_, &[_]) = match property {
        P::Translate => return lower_translate(value, imports),
        P::TransitionDuration | P::AnimationDuration | P::TransitionDelay | P::AnimationDelay => {
            if imports.mode == alien_ir::Mode::Strict {
                return lower_token(value, imports, T::Duration);
            }
            if let Some(time) = numeric_helper(value, imports, "ms")? {
                if time < 0.0 && matches!(property, P::TransitionDuration | P::AnimationDuration) {
                    return Err(at("duration must be non-negative", value.span()));
                }
                return Ok(Value::Milliseconds(time));
            }
            return lower_token(value, imports, T::Duration);
        }
        P::TransitionEase | P::AnimationEase => {
            if named_token_path(value, imports).is_some() {
                return lower_token(value, imports, T::Ease);
            }
            (
                "timingFunction",
                &[
                    ("linear", "linear"),
                    ("ease", "ease"),
                    ("easeIn", "ease-in"),
                    ("easeOut", "ease-out"),
                    ("easeInOut", "ease-in-out"),
                    ("stepStart", "step-start"),
                    ("stepEnd", "step-end"),
                ],
            )
        }
        P::TransitionProperty => return lower_transition_property(value, imports),
        P::AnimationName => {
            if let Some(inner) = custom_argument(value, imports)? {
                let Expression::StringLiteral(string) = inner.get_inner_expression() else {
                    return Err(at("expected an animation name", inner.span()));
                };
                let name = string.value.as_str().filter(|name| is_animation_name(name));
                return name
                    .map(|name| Value::Identifier(name.to_owned()))
                    .ok_or_else(|| at("expected a CSS animation identifier", inner.span()));
            }
            ("animationName", &[("none", "none")])
        }
        P::AnimationIterationCount => {
            if let Some(number) = static_number(value) {
                if number.is_finite() && number >= 0.0 {
                    return Ok(Value::Number(number));
                }
                return Err(at(
                    "expected a finite, non-negative iteration count",
                    value.span(),
                ));
            }
            ("animationIterationCount", &[("infinite", "infinite")])
        }
        P::AnimationDirection => (
            "animationDirection",
            &[
                ("normal", "normal"),
                ("reverse", "reverse"),
                ("alternate", "alternate"),
                ("alternateReverse", "alternate-reverse"),
            ],
        ),
        P::AnimationFillMode => (
            "animationFillMode",
            &[
                ("none", "none"),
                ("forwards", "forwards"),
                ("backwards", "backwards"),
                ("both", "both"),
            ],
        ),
        P::AnimationPlayState => (
            "animationPlayState",
            &[("running", "running"), ("paused", "paused")],
        ),
        _ => unreachable!("only motion properties are lowered here"),
    };
    keyword(value, family, imports, accepted).map(Value::Keyword)
}

fn lower_transition_property(
    value: &Expression<'_>,
    imports: &Imports<'_>,
) -> Result<Value, String> {
    let lower_target = |target: &Expression<'_>| {
        keyword(target, "transitionProperty", imports, TRANSITION_PROPERTIES)
    };
    if matches!(value.get_inner_expression(), Expression::ArrayExpression(_)) {
        return lower_list(value, ListSeparator::Comma, |target| {
            let name = lower_target(target)?;
            if matches!(name, "all" | "none") {
                return Err(at(
                    "transition properties all and none must be used alone",
                    target.span(),
                ));
            }
            Ok(Value::Keyword(name))
        });
    }

    lower_target(value).map(Value::Keyword)
}

fn lower_translate(value: &Expression<'_>, imports: &Imports<'_>) -> Result<Value, String> {
    if css_keyword(value, "translate", imports) == Some("none") {
        return Ok(Value::Keyword("none"));
    }

    let mut axes = [None, None, None];
    for property in &expect_object(value, "translate")?.properties {
        let (key, component) = read_property(property)?;
        let index = match key.as_ref() {
            "x" => 0,
            "y" => 1,
            "z" => 2,
            _ => {
                return Err(at(
                    "expected a translate axis: x, y, or z",
                    component.span(),
                ));
            }
        };
        if axes[index].is_some() {
            return Err(at(&format!("duplicate translate.{key}"), component.span()));
        }
        let component = if index == 2 {
            Value::Length(lower_length(component, imports)?)
        } else {
            lower_dimension(component, imports, true)?
        };
        axes[index] = Some(component);
    }

    let zero = || Value::Number(0.0);
    match axes {
        [x, y, Some(z)] => Ok(Value::List(
            ListSeparator::Space,
            vec![x.unwrap_or_else(zero), y.unwrap_or_else(zero), z],
        )),
        [x, Some(y), None] => Ok(Value::List(
            ListSeparator::Space,
            vec![x.unwrap_or_else(zero), y],
        )),
        [Some(x), None, None] => Ok(x),
        [None, None, None] => Err(at("translate requires at least one axis", value.span())),
    }
}

fn is_animation_name(name: &str) -> bool {
    let name_start = name.strip_prefix('-').unwrap_or(name);
    let mut bytes = name_start.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'-'))
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
        && ![
            "initial",
            "inherit",
            "unset",
            "revert",
            "revert-layer",
            "default",
            "none",
        ]
        .iter()
        .any(|keyword| name.eq_ignore_ascii_case(keyword))
}

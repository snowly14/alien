//! Resolves values and conditions in the IR before writing CSS.

use alien_color::Oklch;
use alien_emit::{CssCondition, CssDeclaration, CssValue};
use alien_ir::{ColorToken, ColorValue, Condition, Declaration, GlobalConfig, TokenFamily, Value};
use alien_tokens::{
    BLACK, BREAKPOINTS, DURATIONS, EASINGS, FONT_SIZES, FONT_WEIGHTS, LINE_HEIGHTS, RADIUS, Radius,
    SHADOWS, SPACE, Shadow, WHITE, color_shade,
};

use crate::CompileError;

pub(crate) fn resolve_condition(condition: &Condition) -> Result<CssCondition, CompileError> {
    let min_width_rem = condition
        .breakpoint
        .as_deref()
        .map(|name| lookup_token(BREAKPOINTS, "breakpoint", name).map(|width| width.0))
        .transpose()?;
    Ok(CssCondition {
        min_width_rem,
        pseudo_classes: condition.pseudo_classes,
    })
}

pub(crate) fn resolve_declaration<'a>(
    declaration: &'a Declaration,
    config: &'a GlobalConfig,
) -> Result<CssDeclaration<'a>, CompileError> {
    Ok(CssDeclaration {
        property: declaration.property,
        value: resolve_value(&declaration.value, config, 0)?,
        important: declaration.important,
    })
}

pub(crate) fn resolve_value<'a>(
    value: &'a Value,
    config: &'a GlobalConfig,
    depth: usize,
) -> Result<CssValue<'a>, CompileError> {
    if depth > 64 {
        return Err(CompileError::Configuration(
            "token reference cycle or expression exceeding 64 levels".to_owned(),
        ));
    }
    Ok(match value {
        Value::Token(family, name) => {
            if let Some(value) = config.tokens.get(&format!("{}.{}", family.as_str(), name)) {
                resolve_value(value, config, depth + 1)?
            } else {
                resolve_token(*family, name)?
            }
        }
        Value::Color(color) => resolve_color_value(color, config, depth + 1)?,
        Value::Length(length) => CssValue::Length(length),
        Value::Percent(value) => CssValue::Percent(*value),
        Value::Fraction(value) => CssValue::Fraction(*value),
        Value::Number(value) => CssValue::Number(*value),
        Value::Milliseconds(value) => CssValue::Milliseconds(*value),
        Value::Keyword(keyword) => CssValue::Text(keyword),
        Value::Identifier(value) => CssValue::Text(value),
        Value::Quoted(value) => CssValue::Quoted(value),
        Value::List(separator, values) => CssValue::List(
            *separator,
            values
                .iter()
                .map(|value| resolve_value(value, config, depth + 1))
                .collect::<Result<_, _>>()?,
        ),
        Value::BorderWidth(width) => CssValue::BorderWidth(width),
        Value::BorderStyle(style) => CssValue::Text(style.as_str()),
    })
}

fn resolve_color_value<'a>(
    color: &'a ColorValue,
    config: &'a GlobalConfig,
    depth: usize,
) -> Result<CssValue<'a>, CompileError> {
    if depth > 64 {
        return Err(CompileError::Configuration(
            "color reference cycle or expression exceeding 64 levels".to_owned(),
        ));
    }
    Ok(match color {
        ColorValue::Token(token) => resolve_color_token(token, config, depth + 1)?,
        ColorValue::Hex(color) => CssValue::Text(color),
        ColorValue::Function { name, channels } => CssValue::ColorFunction(name, *channels),
        ColorValue::WithAlpha {
            color,
            opacity_percent,
        } => CssValue::Alpha(
            Box::new(resolve_color_value(color, config, depth + 1)?),
            *opacity_percent,
        ),
        ColorValue::Mix {
            first,
            second,
            amount,
        } => CssValue::Mix(
            Box::new(resolve_color_value(first, config, depth + 1)?),
            Box::new(resolve_color_value(second, config, depth + 1)?),
            *amount,
        ),
        ColorValue::Alpha {
            token,
            opacity_percent,
        } => match resolve_color_token(token, config, depth + 1)? {
            CssValue::Color(color, _) => CssValue::Color(color, Some(*opacity_percent)),
            color => CssValue::Alpha(Box::new(color), *opacity_percent),
        },
        ColorValue::Keyword(keyword) => CssValue::Text(keyword.as_str()),
    })
}

fn resolve_color_token<'a>(
    token: &ColorToken,
    config: &'a GlobalConfig,
    depth: usize,
) -> Result<CssValue<'a>, CompileError> {
    if let ColorToken::Named(name) = token {
        let value = config.tokens.get(&format!("color.{name}")).ok_or_else(|| {
            CompileError::UnknownToken {
                family: "color",
                name: name.clone(),
            }
        })?;
        return resolve_value(value, config, depth + 1);
    }
    Ok(CssValue::Color(resolve_color(token)?, None))
}

fn resolve_token(family: TokenFamily, name: &str) -> Result<CssValue<'static>, CompileError> {
    let key = family.as_str();
    let value = match family {
        TokenFamily::Space => CssValue::Rem(lookup_token(SPACE, key, name)?.0),
        TokenFamily::Radius => match lookup_token(RADIUS, key, name)? {
            Radius::Fixed(length) => CssValue::Rem(length.0),
            Radius::Full => CssValue::Text("calc(infinity * 1px)"),
        },
        TokenFamily::FontSize => CssValue::Rem(lookup_token(FONT_SIZES, key, name)?.0),
        TokenFamily::FontWeight => {
            CssValue::Number(f64::from(lookup_token(FONT_WEIGHTS, key, name)?))
        }
        TokenFamily::LineHeight => CssValue::Number(lookup_token(LINE_HEIGHTS, key, name)?),
        TokenFamily::Shadow => {
            let Shadow {
                offset_y_px,
                blur_px,
                spread_px,
                opacity_percent,
            } = lookup_token(SHADOWS, key, name)?;
            CssValue::Shadow {
                offset_y_px,
                blur_px,
                spread_px,
                opacity_percent,
            }
        }
        TokenFamily::Duration => {
            CssValue::Milliseconds(f64::from(lookup_token(DURATIONS, key, name)?.0))
        }
        TokenFamily::Ease => CssValue::CubicBezier(lookup_token(EASINGS, key, name)?),
    };
    Ok(value)
}

fn resolve_color(token: &ColorToken) -> Result<Oklch, CompileError> {
    match token {
        ColorToken::Named(name) => Err(CompileError::UnknownToken {
            family: "color",
            name: name.clone(),
        }),
        ColorToken::Black => Ok(BLACK),
        ColorToken::White => Ok(WHITE),
        ColorToken::Shade { palette, step } => {
            color_shade(palette, *step).ok_or_else(|| CompileError::UnknownColorShade {
                palette: palette.clone(),
                step: *step,
            })
        }
    }
}

fn lookup_token<T: Copy>(
    table: &[(&str, T)],
    family: &'static str,
    name: &str,
) -> Result<T, CompileError> {
    table
        .iter()
        .find_map(|&(key, value)| (key == name).then_some(value))
        .ok_or_else(|| CompileError::UnknownToken {
            family,
            name: name.to_owned(),
        })
}

use alien_ir::{ColorKeyword, ColorValue, Mode};
use oxc_ast::ast::Expression;
use oxc_span::{GetSpan, Span};

use super::{
    imports::Imports,
    keyword::css_keyword,
    length::{custom_argument, numeric_helper, static_number},
    token::lower_color_token,
};
use crate::diagnostic::at;

pub(super) fn lower_color(
    value: &Expression<'_>,
    imports: &Imports<'_>,
    property: &str,
) -> Result<ColorValue, String> {
    if let Some(inner) = custom_argument(value, imports)? {
        if imports.mode == Mode::Strict {
            return Err(at(
                "strict mode does not allow custom colors in components; declare a token in alien.globals.ts",
                value.span(),
            ));
        }
        return lower(inner, imports, property, false, 0);
    }
    lower(value, imports, property, imports.mode == Mode::Strict, 0)
}

fn lower(
    value: &Expression<'_>,
    imports: &Imports<'_>,
    property: &str,
    tokens_only: bool,
    depth: usize,
) -> Result<ColorValue, String> {
    if depth > 64 {
        return Err(at(
            "color expressions cannot exceed 64 levels",
            value.span(),
        ));
    }
    if let Some(keyword) = css_keyword(value, "color", imports).and_then(ColorKeyword::from_name) {
        return Ok(ColorValue::Keyword(keyword));
    }
    if matches!(
        value.get_inner_expression(),
        Expression::StaticMemberExpression(_) | Expression::ComputedMemberExpression(_)
    ) {
        return lower_color_token(value, imports, property).map(ColorValue::Token);
    }
    if !tokens_only && let Expression::StringLiteral(string) = value.get_inner_expression() {
        return literal(string.value.as_str(), value);
    }
    let Expression::CallExpression(call) = value.get_inner_expression() else {
        return Err(at(
            &format!(
                "{property} expects a color token imported from `alien` or a supported literal; define custom colors in alien.globals.ts when using strict mode"
            ),
            value.span(),
        ));
    };
    let Expression::Identifier(binding) = call.callee.get_inner_expression() else {
        return Err(at("expected an imported color helper", value.span()));
    };
    let helper = imports.value_export(binding.name.as_str()).unwrap_or("");
    let arguments = call
        .arguments
        .iter()
        .map(|argument| {
            argument
                .as_expression()
                .ok_or_else(|| at("spread arguments are not supported", argument.span()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if helper == "alpha" {
        let [color, percentage] = arguments.as_slice() else {
            return Err(at("alpha() expects a color and a percentage", call.span));
        };
        let opacity_percent = percentage_value(
            percentage,
            imports,
            "alpha() expects percent() between 0 and 100",
        )?;
        let color = lower(color, imports, property, tokens_only, depth + 1)?;
        return Ok(match color {
            ColorValue::Token(token) | ColorValue::Alpha { token, .. } => ColorValue::Alpha {
                token,
                opacity_percent,
            },
            ColorValue::WithAlpha { color, .. } => ColorValue::WithAlpha {
                color,
                opacity_percent,
            },
            color => ColorValue::WithAlpha {
                color: Box::new(color),
                opacity_percent,
            },
        });
    }
    if helper == "mix" {
        let [first, second, amount] = arguments.as_slice() else {
            return Err(at("mix() expects two colors and a percentage", call.span));
        };
        return Ok(ColorValue::Mix {
            first: Box::new(lower(first, imports, property, tokens_only, depth + 1)?),
            second: Box::new(lower(second, imports, property, tokens_only, depth + 1)?),
            amount: percentage_value(amount, imports, "mix() expects percent() between 0 and 100")?,
        });
    }
    if tokens_only {
        return Err(at(
            "strict mode requires color tokens; declare this color in alien.globals.ts",
            value.span(),
        ));
    }
    lower_literal_helper(helper, &arguments, imports, call.span)
}

fn lower_literal_helper(
    helper: &str,
    arguments: &[&Expression<'_>],
    imports: &Imports<'_>,
    span: Span,
) -> Result<ColorValue, String> {
    if helper == "hex" {
        let [argument] = arguments else {
            return Err(at("hex() expects one string", span));
        };
        let Expression::StringLiteral(string) = argument.get_inner_expression() else {
            return Err(at("hex() expects a literal string", argument.span()));
        };
        let text = string.value.as_str();
        if !text.is_some_and(|value| value.starts_with('#')) {
            return Err(at(
                "expected a valid CSS hexadecimal color",
                argument.span(),
            ));
        }
        return literal(text, argument);
    }
    let [first, second, third] = arguments else {
        return Err(at("a color helper expects three components", span));
    };
    let number = |expr: &Expression<'_>| {
        static_number(expr)
            .filter(|number| number.is_finite())
            .ok_or_else(|| at("expected a finite color component", expr.span()))
    };
    let channels = match helper {
        "rgb" => {
            let channels = [number(first)?, number(second)?, number(third)?];
            if !channels.iter().all(|value| (0.0..=255.0).contains(value)) {
                return Err(at("rgb() components must be between 0 and 255", span));
            }
            channels
        }
        "oklch" => {
            let channels = [number(first)?, number(second)?, number(third)?];
            if !(0.0..=1.0).contains(&channels[0]) || channels[1] < 0.0 {
                return Err(at(
                    "oklch() requires lightness between 0 and 1 and non-negative chroma",
                    span,
                ));
            }
            channels
        }
        "hsl" | "hwb" | "hsv" => [
            number(first)?,
            percentage_value(second, imports, "expected percent() between 0 and 100")?,
            percentage_value(third, imports, "expected percent() between 0 and 100")?,
        ],
        _ => return Err(at("unsupported color helper", span)),
    };
    let (name, channels) = match helper {
        "rgb" => ("rgb", channels),
        "oklch" => ("oklch", channels),
        "hsl" => ("hsl", channels),
        "hwb" => ("hwb", channels),
        "hsv" => ("rgb", hsv_to_rgb(channels)),
        _ => unreachable!(),
    };
    Ok(ColorValue::Function { name, channels })
}

fn literal(text: Option<&str>, expression: &Expression<'_>) -> Result<ColorValue, String> {
    let text = text
        .filter(|text| {
            text.split_once('(').is_none_or(|(name, _)| {
                matches!(
                    name.trim().to_ascii_lowercase().as_str(),
                    "rgb" | "rgba" | "hsl" | "hsla" | "hwb" | "lab" | "lch" | "oklab" | "oklch"
                )
            })
        })
        .filter(|value| value.parse::<csscolorparser::Color>().is_ok())
        .ok_or_else(|| {
            at(
                "expected a valid CSS hexadecimal color or supported CSS color literal",
                expression.span(),
            )
        })?;
    Ok(ColorValue::Hex(text.to_owned()))
}

fn percentage_value(
    value: &Expression<'_>,
    imports: &Imports<'_>,
    message: &str,
) -> Result<f64, String> {
    numeric_helper(value, imports, "percent")?
        .filter(|value| (0.0..=100.0).contains(value))
        .ok_or_else(|| at(message, value.span()))
}

fn hsv_to_rgb([hue, saturation, value]: [f64; 3]) -> [f64; 3] {
    let hue = hue.rem_euclid(360.0) / 60.0;
    let value = value / 100.0;
    let chroma = value * saturation / 100.0;
    let secondary = chroma * (1.0 - (hue % 2.0 - 1.0).abs());
    let rgb = if hue < 1.0 {
        [chroma, secondary, 0.0]
    } else if hue < 2.0 {
        [secondary, chroma, 0.0]
    } else if hue < 3.0 {
        [0.0, chroma, secondary]
    } else if hue < 4.0 {
        [0.0, secondary, chroma]
    } else if hue < 5.0 {
        [secondary, 0.0, chroma]
    } else {
        [chroma, 0.0, secondary]
    };
    rgb.map(|component| (component + value - chroma) * 255.0)
}

//! Converts a supported style property and its value into a declaration.

use alien_ir::{Mode, Property as P, TokenFamily as T, Value};
use oxc_ast::ast::Expression;
use oxc_span::GetSpan;

use super::{
    border, color,
    imports::Imports,
    keyword::css_keyword,
    length::{custom_argument, lower_dimension, static_number},
    motion, standard,
    token::{lower_token, named_token_path},
};
use crate::diagnostic::at;

pub(super) fn lower_value(
    property: P,
    value: &Expression<'_>,
    imports: &Imports<'_>,
) -> Result<Value, String> {
    match property {
        P::BackgroundColor
        | P::Color
        | P::OutlineColor
        | P::TextDecorationColor
        | P::BorderColor
        | P::BorderTopColor
        | P::BorderRightColor
        | P::BorderBottomColor
        | P::BorderLeftColor => {
            color::lower_color(value, imports, property.source_name()).map(Value::Color)
        }
        P::Padding
        | P::PaddingTop
        | P::PaddingRight
        | P::PaddingBottom
        | P::PaddingLeft
        | P::PaddingBlock
        | P::PaddingInline => lower_spacing(value, imports, None, false),
        P::Margin
        | P::MarginTop
        | P::MarginRight
        | P::MarginBottom
        | P::MarginLeft
        | P::MarginBlock
        | P::MarginInline => lower_spacing(value, imports, Some(("margin", "auto")), true),
        P::Gap | P::RowGap | P::ColumnGap => {
            let family = property.source_name();
            if css_keyword(value, family, imports) == Some("normal") {
                Ok(Value::Keyword("normal"))
            } else {
                lower_spacing(value, imports, Some(("gap", "normal")), false)
            }
        }
        P::BorderRadius
        | P::BorderTopLeftRadius
        | P::BorderTopRightRadius
        | P::BorderBottomRightRadius
        | P::BorderBottomLeftRadius => {
            if imports.mode == Mode::Strict {
                return lower_token(value, imports, T::Radius);
            }
            if named_token_path(value, imports).is_some() {
                lower_token(value, imports, T::Radius)
            } else {
                lower_dimension(
                    custom_argument(value, imports)?.unwrap_or(value),
                    imports,
                    false,
                )
            }
        }
        P::BorderWidth
        | P::BorderTopWidth
        | P::BorderRightWidth
        | P::BorderBottomWidth
        | P::BorderLeftWidth => {
            border::lower_width(value, "borderWidth", imports).map(Value::BorderWidth)
        }
        P::BorderStyle
        | P::BorderTopStyle
        | P::BorderRightStyle
        | P::BorderBottomStyle
        | P::BorderLeftStyle => border::lower_style(value, imports).map(Value::BorderStyle),
        P::FontSize => {
            if imports.mode == Mode::Strict || named_token_path(value, imports).is_some() {
                lower_token(value, imports, T::FontSize)
            } else {
                lower_dimension(value, imports, false)
            }
        }
        P::FontWeight => lower_font_weight(value, imports),
        P::LineHeight => lower_line_height(value, imports),
        P::BoxShadow => lower_token(value, imports, T::Shadow),
        P::Translate
        | P::TransitionProperty
        | P::TransitionDuration
        | P::TransitionDelay
        | P::TransitionEase
        | P::AnimationName
        | P::AnimationDuration
        | P::AnimationDelay
        | P::AnimationEase
        | P::AnimationIterationCount
        | P::AnimationDirection
        | P::AnimationFillMode
        | P::AnimationPlayState => motion::lower_value(property, value, imports),
        _ => standard::lower_value(property, value, imports),
    }
}

fn lower_spacing(
    value: &Expression<'_>,
    imports: &Imports<'_>,
    keyword: Option<(&str, &'static str)>,
    allow_negative: bool,
) -> Result<Value, String> {
    if let Some((family, name)) = keyword
        && css_keyword(value, family, imports) == Some(name)
    {
        return Ok(Value::Keyword(name));
    }
    if imports.mode == Mode::Strict || named_token_path(value, imports).is_some() {
        return lower_token(value, imports, T::Space);
    }
    lower_dimension(
        custom_argument(value, imports)?.unwrap_or(value),
        imports,
        allow_negative,
    )
}

fn lower_font_weight(value: &Expression<'_>, imports: &Imports<'_>) -> Result<Value, String> {
    if let Some(keyword) = css_keyword(value, "fontWeight", imports) {
        return match keyword {
            "normal" => Ok(Value::Keyword("normal")),
            "bold" => Ok(Value::Keyword("bold")),
            "bolder" => Ok(Value::Keyword("bolder")),
            "lighter" => Ok(Value::Keyword("lighter")),
            _ => Err(at("expected a font weight keyword", value.span())),
        };
    }
    if imports.mode == Mode::Strict {
        return lower_token(value, imports, T::FontWeight);
    }
    if named_token_path(value, imports).is_some() {
        return lower_token(value, imports, T::FontWeight);
    }
    {
        let inner = custom_argument(value, imports)?.unwrap_or(value);
        static_number(inner)
            .filter(|value| (1.0..=1000.0).contains(value))
            .map(Value::Number)
            .ok_or_else(|| at("font weight must be between 1 and 1000", inner.span()))
    }
}

fn lower_line_height(value: &Expression<'_>, imports: &Imports<'_>) -> Result<Value, String> {
    if css_keyword(value, "lineHeight", imports) == Some("normal") {
        return Ok(Value::Keyword("normal"));
    }
    if imports.mode == Mode::Strict || named_token_path(value, imports).is_some() {
        return lower_token(value, imports, T::LineHeight);
    }
    if let Some(number) = static_number(value) {
        if number.is_finite() && number >= 0.0 {
            return Ok(Value::Number(number));
        }
        return Err(at(
            "expected a finite, non-negative line height",
            value.span(),
        ));
    }
    lower_dimension(value, imports, false)
}

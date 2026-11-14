use alien_ir::{ListSeparator, Property as P, Value};
use oxc_ast::ast::Expression;
use oxc_span::GetSpan;

use super::{
    border,
    imports::Imports,
    keyword::{css_keyword, standard_keyword},
    length::{custom_argument, lower_dimension, lower_length, numeric_helper, static_number},
    object::lower_list,
};
use crate::diagnostic::at;

const AUTO: &[&str] = &["auto"];
const NORMAL: &[&str] = &["normal"];
const INTRINSIC_SIZE: &[&str] = &["min-content", "max-content", "fit-content"];
const POSITIONS: &[&str] = &["top", "right", "bottom", "left", "center"];
const OVERFLOW: &[&str] = &["visible", "hidden", "clip", "auto", "scroll"];
const ALIGN_POSITION: &[&str] = &[
    "normal",
    "center",
    "start",
    "end",
    "flex-start",
    "flex-end",
    "stretch",
];
const ALIGN_ITEMS: &[&str] = &["baseline", "self-start", "self-end"];
const DISTRIBUTION: &[&str] = &["space-between", "space-around", "space-evenly"];
const CURSORS: &[&str] = &[
    "auto",
    "default",
    "none",
    "context-menu",
    "help",
    "pointer",
    "progress",
    "wait",
    "cell",
    "crosshair",
    "text",
    "vertical-text",
    "alias",
    "copy",
    "move",
    "no-drop",
    "not-allowed",
    "grab",
    "grabbing",
    "all-scroll",
    "col-resize",
    "row-resize",
    "n-resize",
    "e-resize",
    "s-resize",
    "w-resize",
    "ne-resize",
    "nw-resize",
    "se-resize",
    "sw-resize",
    "ew-resize",
    "ns-resize",
    "nesw-resize",
    "nwse-resize",
    "zoom-in",
    "zoom-out",
];

pub(super) fn lower_value(
    property: P,
    value: &Expression<'_>,
    imports: &Imports<'_>,
) -> Result<Value, String> {
    let family = property.source_name();
    match property {
        P::Width | P::Height | P::MinWidth | P::MinHeight => {
            dimension_or_keyword(value, imports, family, &[AUTO, INTRINSIC_SIZE], false)
        }
        P::MaxWidth | P::MaxHeight => {
            dimension_or_keyword(value, imports, family, &[&["none"], INTRINSIC_SIZE], false)
        }
        P::FlexBasis => dimension_or_keyword(
            value,
            imports,
            family,
            &[AUTO, &["content"], INTRINSIC_SIZE],
            false,
        ),
        P::Inset | P::Top | P::Right | P::Bottom | P::Left => {
            dimension_or_keyword(value, imports, "inset", &[AUTO], true)
        }
        P::BackgroundSize => dimension_or_keyword(
            value,
            imports,
            family,
            &[AUTO, &["cover", "contain"]],
            false,
        ),
        P::BackgroundPosition | P::ObjectPosition => {
            dimension_or_keyword(value, imports, family, &[POSITIONS], true)
        }
        P::LetterSpacing if css_keyword(value, family, imports).is_some() => {
            standard_keyword(value, family, imports, &[NORMAL]).map(Value::Keyword)
        }
        P::LetterSpacing | P::OutlineOffset => lower_length(value, imports).map(Value::Length),
        P::OutlineWidth => {
            border::lower_width(value, "outlineWidth", imports).map(Value::BorderWidth)
        }
        P::AspectRatio => {
            number_or_keyword(value, imports, family, &[AUTO], |number| number >= 0.0)
        }
        P::Flex => number_or_keyword(value, imports, family, &[AUTO, &["none"]], |number| {
            number >= 0.0
        }),
        P::FlexGrow | P::FlexShrink => {
            number_or_keyword(value, imports, family, &[], |number| number >= 0.0)
        }
        P::ZIndex => number_or_keyword(value, imports, family, &[AUTO], is_integer),
        P::Order => number_or_keyword(value, imports, family, &[], is_integer),
        P::GridColumn | P::GridRow => {
            number_or_keyword(value, imports, family, &[AUTO], |number| {
                is_integer(number) && number != 0.0
            })
        }
        P::Opacity => match numeric_helper(value, imports, "percent")? {
            Some(percent) => Ok(Value::Percent(percent)),
            None => number_or_keyword(value, imports, family, &[], |_| true),
        },
        P::GridTemplateColumns | P::GridTemplateRows
            if css_keyword(value, family, imports).is_some() =>
        {
            standard_keyword(value, family, imports, &[&["none", "subgrid"]]).map(Value::Keyword)
        }
        P::GridTemplateColumns | P::GridTemplateRows | P::GridAutoColumns | P::GridAutoRows => {
            lower_list(value, ListSeparator::Space, |track| {
                lower_grid_track(track, imports)
            })
        }
        P::FontFamily => {
            if matches!(value.get_inner_expression(), Expression::ArrayExpression(_)) {
                lower_list(value, ListSeparator::Comma, |font| {
                    lower_font(font, imports)
                })
            } else {
                lower_font(value, imports)
            }
        }
        _ => lower_keyword_property(property, value, imports),
    }
}

fn lower_keyword_property(
    property: P,
    value: &Expression<'_>,
    imports: &Imports<'_>,
) -> Result<Value, String> {
    let allowed: &[&[&str]] = match property {
        P::Display => &[&[
            "none",
            "contents",
            "block",
            "inline",
            "inline-block",
            "flow-root",
            "flex",
            "inline-flex",
            "grid",
            "inline-grid",
            "table",
            "inline-table",
            "list-item",
        ]],
        P::Position => &[&["static", "relative", "absolute", "fixed", "sticky"]],
        P::BoxSizing => &[&["content-box", "border-box"]],
        P::Overflow | P::OverflowX | P::OverflowY => &[OVERFLOW],
        P::FlexDirection => &[&["row", "row-reverse", "column", "column-reverse"]],
        P::FlexWrap => &[&["nowrap", "wrap", "wrap-reverse"]],
        P::AlignItems => &[ALIGN_POSITION, ALIGN_ITEMS],
        P::AlignSelf => &[AUTO, ALIGN_POSITION, ALIGN_ITEMS],
        P::AlignContent => &[ALIGN_POSITION, DISTRIBUTION, &["baseline"]],
        P::JustifyContent => &[ALIGN_POSITION, DISTRIBUTION, &["left", "right"]],
        P::JustifyItems => &[ALIGN_POSITION, ALIGN_ITEMS, &["left", "right"]],
        P::JustifySelf => &[AUTO, ALIGN_POSITION, ALIGN_ITEMS, &["left", "right"]],
        P::GridAutoFlow => &[&["row", "column", "dense", "row dense", "column dense"]],
        P::FontStyle => &[&["normal", "italic", "oblique"]],
        P::TextAlign => &[&[
            "start",
            "end",
            "left",
            "right",
            "center",
            "justify",
            "match-parent",
        ]],
        P::TextDecorationLine => &[&["none", "underline", "overline", "line-through"]],
        P::TextDecorationStyle => &[&["solid", "double", "dotted", "dashed", "wavy"]],
        P::TextOverflow => &[&["clip", "ellipsis"]],
        P::TextTransform => &[&["none", "capitalize", "uppercase", "lowercase"]],
        P::WhiteSpace => &[&[
            "normal",
            "nowrap",
            "pre",
            "pre-wrap",
            "pre-line",
            "break-spaces",
        ]],
        P::OverflowWrap => &[&["normal", "anywhere", "break-word"]],
        P::WordBreak => &[&["normal", "break-all", "keep-all"]],
        P::BackgroundRepeat => &[&[
            "repeat",
            "no-repeat",
            "repeat-x",
            "repeat-y",
            "round",
            "space",
        ]],
        P::ObjectFit => &[&["fill", "contain", "cover", "none", "scale-down"]],
        P::OutlineStyle => &[&[
            "auto", "none", "dotted", "dashed", "solid", "double", "groove", "ridge", "inset",
            "outset",
        ]],
        P::Visibility => &[&["visible", "hidden", "collapse"]],
        P::Cursor => &[CURSORS],
        P::PointerEvents | P::Appearance => &[&["auto", "none"]],
        P::UserSelect => &[&["auto", "none", "text", "all"]],
        _ => {
            return Err(at(
                &format!("unsupported style property `{}`", property.source_name()),
                value.span(),
            ));
        }
    };
    standard_keyword(value, property.source_name(), imports, allowed).map(Value::Keyword)
}

fn dimension_or_keyword(
    value: &Expression<'_>,
    imports: &Imports<'_>,
    family: &str,
    accepted: &[&[&'static str]],
    allow_negative: bool,
) -> Result<Value, String> {
    if let Ok(keyword) = standard_keyword(value, family, imports, accepted) {
        return Ok(Value::Keyword(keyword));
    }
    lower_dimension(value, imports, allow_negative)
}

fn number_or_keyword(
    value: &Expression<'_>,
    imports: &Imports<'_>,
    family: &str,
    accepted: &[&[&'static str]],
    valid: impl FnOnce(f64) -> bool,
) -> Result<Value, String> {
    if !accepted.is_empty() && css_keyword(value, family, imports).is_some() {
        return standard_keyword(value, family, imports, accepted).map(Value::Keyword);
    }
    static_number(value)
        .filter(|number| number.is_finite() && valid(*number))
        .map(Value::Number)
        .ok_or_else(|| {
            at(
                &format!("invalid numeric value for `{family}`"),
                value.span(),
            )
        })
}

fn is_integer(number: f64) -> bool {
    number.fract() == 0.0
}

fn lower_grid_track(value: &Expression<'_>, imports: &Imports<'_>) -> Result<Value, String> {
    if let Some(fraction) = numeric_helper(value, imports, "fr")? {
        if fraction < 0.0 {
            return Err(at("grid fractions must be non-negative", value.span()));
        }
        return Ok(Value::Fraction(fraction));
    }
    dimension_or_keyword(
        value,
        imports,
        "gridTrack",
        &[AUTO, &["min-content", "max-content"]],
        false,
    )
}

fn lower_font(value: &Expression<'_>, imports: &Imports<'_>) -> Result<Value, String> {
    if let Some(inner) = custom_argument(value, imports)? {
        if let Expression::StringLiteral(string) = inner.get_inner_expression()
            && let Some(name) = string.value.as_str()
        {
            return Ok(Value::Quoted(name.to_owned()));
        }
        return Err(at("expected a font family name", inner.span()));
    }
    standard_keyword(
        value,
        "fontFamily",
        imports,
        &[&[
            "serif",
            "sans-serif",
            "monospace",
            "system-ui",
            "cursive",
            "fantasy",
        ]],
    )
    .map(Value::Keyword)
}

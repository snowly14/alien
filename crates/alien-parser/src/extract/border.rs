use alien_ir::{BorderStyle, BorderWidth, Length};
use oxc_ast::ast::Expression;
use oxc_span::GetSpan;

use super::{imports::Imports, keyword::css_keyword, length::lower_length};
use crate::diagnostic::at;

pub(super) fn lower_style(
    value: &Expression<'_>,
    imports: &Imports<'_>,
) -> Result<BorderStyle, String> {
    css_keyword(value, "borderStyle", imports)
        .and_then(BorderStyle::from_keyword)
        .ok_or_else(|| at("expected a border style keyword", value.span()))
}

pub(super) fn lower_width(
    value: &Expression<'_>,
    family: &str,
    imports: &Imports<'_>,
) -> Result<BorderWidth, String> {
    if let Some(width) = css_keyword(value, family, imports).and_then(|name| match name {
        "thin" => Some(BorderWidth::Thin),
        "medium" => Some(BorderWidth::Medium),
        "thick" => Some(BorderWidth::Thick),
        _ => None,
    }) {
        return Ok(width);
    }
    let length = lower_length(value, imports)?;
    if matches!(length, Length::Literal { value, .. } if value < 0.0) {
        return Err(at("width must be non-negative", value.span()));
    }
    Ok(BorderWidth::Length(length))
}

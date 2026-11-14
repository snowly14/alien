use std::fmt::Write;

use alien_color::Oklch;
use alien_ir::{BorderWidth, Length, ListSeparator, Property};

use super::length::write_length;

/// A CSS property with its resolved value.
pub struct CssDeclaration<'value> {
    pub property: Property,
    pub value: CssValue<'value>,
    pub important: bool,
}

/// A resolved value ready for CSS serialization.
pub enum CssValue<'value> {
    Color(Oklch, Option<f64>),
    ColorFunction(&'static str, [f64; 3]),
    Alpha(Box<Self>, f64),
    Mix(Box<Self>, Box<Self>, f64),
    Rem(f32),
    Number(f64),
    Percent(f64),
    Fraction(f64),
    Milliseconds(f64),
    Length(&'value Length),
    BorderWidth(&'value BorderWidth),
    Text(&'value str),
    Quoted(&'value str),
    List(ListSeparator, Vec<Self>),
    CubicBezier([f32; 4]),
    Shadow {
        offset_y_px: u8,
        blur_px: u8,
        spread_px: i8,
        opacity_percent: u8,
    },
}

impl CssDeclaration<'_> {
    pub(super) fn write_to(&self, output: &mut String) {
        output.push_str(self.property.as_css());
        output.push(':');
        self.value.write_to(output);
        if self.important {
            output.push_str("!important");
        }
    }
}

impl CssValue<'_> {
    fn write_to(&self, output: &mut String) {
        match self {
            CssValue::ColorFunction(name, [a, b, c]) => {
                if matches!(*name, "hsl" | "hwb") {
                    let _ = write!(output, "{name}({a} {b}% {c}%)");
                } else {
                    let _ = write!(output, "{name}({a} {b} {c})");
                }
            }
            CssValue::Alpha(color, opacity) => {
                output.push_str("oklch(from ");
                color.write_to(output);
                let _ = write!(output, " l c h/{opacity}%)");
            }
            CssValue::Mix(first, second, amount) => {
                output.push_str("color-mix(in oklch,");
                first.write_to(output);
                output.push(',');
                second.write_to(output);
                let _ = write!(output, " {amount}%)");
            }
            CssValue::Color(color, opacity_percent) => {
                let _ = color.write_css(output, *opacity_percent);
            }
            CssValue::Rem(0.0) => output.push('0'),
            CssValue::Rem(value) => {
                let _ = write!(output, "{value}rem");
            }
            CssValue::Number(value) => {
                let _ = write!(output, "{value}");
            }
            CssValue::Percent(value) => {
                let _ = write!(output, "{value}%");
            }
            CssValue::Fraction(value) => {
                let _ = write!(output, "{value}fr");
            }
            CssValue::Milliseconds(value) => {
                let _ = write!(output, "{value}ms");
            }
            CssValue::Length(length) => write_length(output, length),
            CssValue::BorderWidth(width) => match width {
                BorderWidth::Thin => output.push_str("thin"),
                BorderWidth::Medium => output.push_str("medium"),
                BorderWidth::Thick => output.push_str("thick"),
                BorderWidth::Length(length) => write_length(output, length),
            },
            CssValue::Text(value) => output.push_str(value),
            CssValue::Quoted(value) => write_css_string(output, value),
            CssValue::List(separator, values) => {
                if let Some((first, remaining)) = values.split_first() {
                    first.write_to(output);
                    for value in remaining {
                        output.push(separator.as_char());
                        value.write_to(output);
                    }
                }
            }
            CssValue::CubicBezier([x1, y1, x2, y2]) => {
                let _ = write!(output, "cubic-bezier({x1},{y1},{x2},{y2})");
            }
            CssValue::Shadow {
                offset_y_px,
                blur_px,
                spread_px,
                opacity_percent,
            } => {
                let _ = write!(output, "0 {offset_y_px}px {blur_px}px");
                if *spread_px != 0 {
                    let _ = write!(output, " {spread_px}px");
                }
                let _ = write!(output, " rgb(0 0 0/{opacity_percent}%)");
            }
        }
    }
}

fn write_css_string(output: &mut String, value: &str) {
    output.push('"');
    for character in value.chars() {
        match character {
            '"' | '\\' => {
                output.push('\\');
                output.push(character);
            }
            '\0' => output.push('\u{fffd}'),
            character if character.is_ascii_control() => {
                let _ = write!(output, "\\{:x} ", u32::from(character));
            }
            character => output.push(character),
        }
    }
    output.push('"');
}

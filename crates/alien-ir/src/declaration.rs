use crate::{BorderStyle, BorderWidth, ColorValue, Length, Property};

/// A property paired with a statically extracted value.
#[derive(Debug)]
pub struct Declaration {
    pub property: Property,
    pub value: Value,
    pub important: bool,
}

/// A value supported by a compiled style declaration.
#[derive(Debug)]
pub enum Value {
    Token(TokenFamily, String),
    Color(ColorValue),
    Length(Length),
    Percent(f64),
    Fraction(f64),
    Number(f64),
    Milliseconds(f64),
    Keyword(&'static str),
    Identifier(String),
    Quoted(String),
    List(ListSeparator, Vec<Self>),
    BorderWidth(BorderWidth),
    BorderStyle(BorderStyle),
}

/// Separates the values of a CSS list.
#[derive(Debug, Clone, Copy)]
pub enum ListSeparator {
    Space,
    Comma,
}

impl ListSeparator {
    #[must_use]
    pub const fn as_char(self) -> char {
        match self {
            Self::Space => ' ',
            Self::Comma => ',',
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TokenFamily {
    Space,
    Radius,
    FontSize,
    FontWeight,
    LineHeight,
    Shadow,
    Duration,
    Ease,
}

impl TokenFamily {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Space => "space",
            Self::Radius => "radius",
            Self::FontSize => "fontSize",
            Self::FontWeight => "fontWeight",
            Self::LineHeight => "lineHeight",
            Self::Shadow => "shadow",
            Self::Duration => "duration",
            Self::Ease => "ease",
        }
    }
}

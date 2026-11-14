/// A reference to a built-in color.
#[derive(Debug)]
pub enum ColorToken {
    Named(String),
    Black,
    White,
    Shade { palette: String, step: u16 },
}

/// A color reference, optionally with opacity, or a CSS color keyword.
#[derive(Debug)]
pub enum ColorValue {
    Token(ColorToken),
    Hex(String),
    Function {
        name: &'static str,
        channels: [f64; 3],
    },
    WithAlpha {
        color: Box<Self>,
        opacity_percent: f64,
    },
    Mix {
        first: Box<Self>,
        second: Box<Self>,
        amount: f64,
    },
    Alpha {
        token: ColorToken,
        opacity_percent: f64,
    },
    Keyword(ColorKeyword),
}

/// A CSS color keyword supported by the compiler.
#[derive(Debug, Clone, Copy)]
pub enum ColorKeyword {
    CurrentColor,
    Transparent,
}

impl ColorKeyword {
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "currentColor" => Some(Self::CurrentColor),
            "transparent" => Some(Self::Transparent),
            _ => None,
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CurrentColor => "currentColor",
            Self::Transparent => "transparent",
        }
    }
}

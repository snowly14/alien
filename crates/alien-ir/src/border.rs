use crate::Length;

/// The width of a border.
#[derive(Debug, Clone)]
pub enum BorderWidth {
    Thin,
    Medium,
    Thick,
    Length(Length),
}

/// A validated border style keyword.
#[derive(Debug, Clone, Copy)]
pub struct BorderStyle(&'static str);

impl BorderStyle {
    #[must_use]
    pub fn from_keyword(name: &str) -> Option<Self> {
        const KEYWORDS: [&str; 10] = [
            "none", "hidden", "dotted", "dashed", "solid", "double", "groove", "ridge", "inset",
            "outset",
        ];
        KEYWORDS
            .into_iter()
            .find(|keyword| *keyword == name)
            .map(Self)
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

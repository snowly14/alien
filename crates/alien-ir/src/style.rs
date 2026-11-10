/// A named style extracted from an Alien source file.
#[derive(Debug)]
pub struct Style {
    pub name: String,
    pub rules: Vec<Rule>,
}

/// A declaration and the condition under which it applies.
#[derive(Debug)]
pub struct Rule {
    pub condition: Condition,
    pub declaration: Declaration,
}

/// The conditions supported by the first compiler pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Condition {
    Base,
    Hover,
}

/// A typed style declaration.
#[derive(Debug)]
pub enum Declaration {
    BackgroundColor(ColorToken),
}

/// A reference to a built-in color.
#[derive(Debug)]
pub enum ColorToken {
    Black,
    White,
    Shade { palette: String, step: u16 },
}

use crate::{Condition, Declaration};

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

mod pseudo_class;

pub use pseudo_class::{PseudoClass, PseudoClasses};

/// The condition under which a declaration applies.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Condition {
    pub breakpoint: Option<String>,
    pub pseudo_classes: PseudoClasses,
}

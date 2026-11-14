use std::collections::BTreeMap;

use crate::Value;

/// Component design-system policy. CSS keywords remain available in both modes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Free,
    Strict,
}

/// Optional CSS reset, independent of the design-system policy.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Reset {
    #[default]
    Disabled,
    Light,
    Medium,
    Strong,
}

/// Statically parsed project configuration with tokens keyed by `family.name`.
#[derive(Debug, Default)]
pub struct GlobalConfig {
    pub mode: Mode,
    pub reset: Reset,
    pub tokens: BTreeMap<String, Value>,
}

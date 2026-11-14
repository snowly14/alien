use alien_core::units::Milliseconds;

/// Built-in motion durations.
pub const DURATIONS: &[(&str, Milliseconds)] = &[
    ("fast", Milliseconds(120)),
    ("normal", Milliseconds(180)),
    ("slow", Milliseconds(300)),
];

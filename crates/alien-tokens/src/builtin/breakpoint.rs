use alien_core::units::Rem;

/// Built-in minimum viewport widths.
pub const BREAKPOINTS: &[(&str, Rem)] = &[
    ("sm", Rem(40.0)),
    ("md", Rem(48.0)),
    ("lg", Rem(64.0)),
    ("xl", Rem(80.0)),
    ("xl2", Rem(96.0)),
];

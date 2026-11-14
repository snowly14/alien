use alien_core::units::Rem;

/// Built-in spacing values.
pub const SPACE: &[(&str, Rem)] = &[
    ("none", Rem(0.0)),
    ("xxs", Rem(0.125)),
    ("xs", Rem(0.25)),
    ("sm", Rem(0.5)),
    ("md", Rem(1.0)),
    ("lg", Rem(1.5)),
    ("xl", Rem(2.0)),
    ("xl2", Rem(3.0)),
    ("xl3", Rem(4.0)),
    ("xl4", Rem(5.0)),
    ("xl5", Rem(6.0)),
    ("xl6", Rem(8.0)),
    ("xl7", Rem(10.0)),
];

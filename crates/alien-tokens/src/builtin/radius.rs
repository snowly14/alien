use alien_core::units::Rem;

/// A built-in corner radius.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Radius {
    /// A fixed radius.
    Fixed(Rem),
    /// The maximum rounding allowed by the element's dimensions.
    Full,
}

/// Built-in corner radius tokens, following the Tailwind CSS scale.
pub const RADIUS: &[(&str, Radius)] = &[
    ("none", Radius::Fixed(Rem(0.0))),
    ("xs", Radius::Fixed(Rem(0.125))),
    ("sm", Radius::Fixed(Rem(0.25))),
    ("md", Radius::Fixed(Rem(0.375))),
    ("lg", Radius::Fixed(Rem(0.5))),
    ("xl", Radius::Fixed(Rem(0.75))),
    ("xl2", Radius::Fixed(Rem(1.0))),
    ("xl3", Radius::Fixed(Rem(1.5))),
    ("xl4", Radius::Fixed(Rem(2.0))),
    ("full", Radius::Full),
];

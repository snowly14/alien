//! Shared CSS units.

/// A length expressed in rem.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rem(pub f32);

/// A duration expressed in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Milliseconds(pub u16);

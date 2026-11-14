/// A supported CSS length unit.
#[derive(Debug, Clone, Copy)]
pub struct LengthUnit(&'static str);

impl LengthUnit {
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        const UNITS: [&str; 8] = ["px", "rem", "em", "ch", "vw", "vh", "svh", "dvh"];
        UNITS.into_iter().find(|unit| *unit == name).map(Self)
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

/// A length literal or a calculation between lengths.
#[derive(Debug, Clone)]
pub enum Length {
    Literal { value: f64, unit: LengthUnit },
    Sub { left: Box<Self>, right: Box<Self> },
}

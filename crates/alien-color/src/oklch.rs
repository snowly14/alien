use std::fmt;

/// A validated color in the OKLCH color space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Oklch {
    lightness: f32,
    chroma: f32,
    hue: Option<f32>,
}

impl Oklch {
    /// Creates a color with lightness in 0..=1, non-negative chroma, and a finite hue in degrees.
    #[must_use]
    pub const fn new(lightness: f32, chroma: f32, hue: f32) -> Option<Self> {
        if lightness >= 0.0
            && lightness <= 1.0
            && chroma >= 0.0
            && chroma.is_finite()
            && hue.is_finite()
        {
            Some(Self {
                lightness,
                chroma,
                hue: Some(hue),
            })
        } else {
            None
        }
    }

    /// Returns the lightness component.
    #[must_use]
    pub const fn lightness(self) -> f32 {
        self.lightness
    }

    /// Returns the chroma component.
    #[must_use]
    pub const fn chroma(self) -> f32 {
        self.chroma
    }

    /// Creates a color with a missing hue.
    #[must_use]
    pub const fn without_hue(lightness: f32, chroma: f32) -> Option<Self> {
        match Self::new(lightness, chroma, 0.0) {
            Some(color) => Some(Self { hue: None, ..color }),
            None => None,
        }
    }

    /// Returns the hue in degrees, using zero for a missing hue.
    #[must_use]
    pub const fn hue(self) -> f32 {
        match self.hue {
            Some(hue) => hue,
            None => 0.0,
        }
    }

    /// Returns the hue component, preserving a missing hue.
    #[must_use]
    pub const fn hue_component(self) -> Option<f32> {
        self.hue
    }
}

impl fmt::Display for Oklch {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.hue {
            Some(hue) => write!(
                formatter,
                "oklch({} {} {})",
                self.lightness, self.chroma, hue
            ),
            None => write!(formatter, "oklch({} {} none)", self.lightness, self.chroma),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Oklch;

    #[test]
    fn validates_components() {
        assert!(Oklch::new(0.5, 0.2, -30.0).is_some());
        assert!(Oklch::new(-0.1, 0.2, 0.0).is_none());
        assert!(Oklch::new(1.1, 0.2, 0.0).is_none());
        assert!(Oklch::new(0.5, -0.1, 0.0).is_none());
        assert!(Oklch::new(f32::NAN, 0.2, 0.0).is_none());
        assert!(Oklch::new(0.5, f32::INFINITY, 0.0).is_none());
        assert!(Oklch::new(0.5, 0.2, f32::INFINITY).is_none());
    }

    #[test]
    fn preserves_missing_hue() {
        let color = Oklch::without_hue(0.985, 0.0).unwrap();
        assert_eq!(color.hue_component(), None);
        assert_eq!(color.hue(), 0.0);
        assert_eq!(color.to_string(), "oklch(0.985 0 none)");
        assert!(Oklch::without_hue(f32::NAN, 0.0).is_none());
    }

    #[test]
    fn displays_components_without_fixed_rounding() {
        let color = Oklch::new(0.6056, 0.2189, 292.72).unwrap();
        assert_eq!(color.to_string(), "oklch(0.6056 0.2189 292.72)");
    }
}

use alien_color::Oklch;

/// A numbered shade in a built-in palette.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shade {
    pub step: u16,
    pub color: Oklch,
}

/// A named built-in palette. Shades are ordered from light to dark.
#[derive(Debug)]
pub struct ColorPalette {
    pub name: &'static str,
    pub shades: &'static [Shade],
}

const fn oklch(lightness: f32, chroma: f32, hue: f32) -> Oklch {
    let Some(color) = Oklch::new(lightness, chroma, hue) else {
        panic!("invalid built-in OKLCH color");
    };
    color
}

const fn shade(step: u16, lightness: f32, chroma: f32, hue: f32) -> Shade {
    Shade {
        step,
        color: oklch(lightness, chroma, hue),
    }
}

const fn shade_without_hue(step: u16, lightness: f32, chroma: f32) -> Shade {
    let Some(color) = Oklch::without_hue(lightness, chroma) else {
        panic!("invalid built-in OKLCH color");
    };
    Shade { step, color }
}

mod palettes;

pub use palettes::COLOR_PALETTES;

/// The built-in black color.
pub const BLACK: Oklch = oklch(0.0, 0.0, 0.0);

/// The built-in white color.
pub const WHITE: Oklch = oklch(1.0, 0.0, 0.0);

/// Returns a built-in color by palette name and shade step.
#[must_use]
pub fn color_shade(palette: &str, step: u16) -> Option<Oklch> {
    let palette = COLOR_PALETTES.iter().find(|entry| entry.name == palette)?;
    let index = palette
        .shades
        .binary_search_by_key(&step, |shade| shade.step)
        .ok()?;
    Some(palette.shades[index].color)
}

#[cfg(test)]
mod tests {
    use super::{BLACK, COLOR_PALETTES, WHITE};

    #[test]
    fn palettes_have_unique_names_and_ordered_shades() {
        assert_eq!(COLOR_PALETTES.len(), 26);
        let expected_steps = [50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950];
        for (index, palette) in COLOR_PALETTES.iter().enumerate() {
            assert!(
                palette
                    .shades
                    .iter()
                    .map(|shade| shade.step)
                    .eq(expected_steps),
                "unexpected shade steps in palette {}",
                palette.name
            );
            assert_ne!(palette.name, "");
            assert!(
                COLOR_PALETTES[..index]
                    .iter()
                    .all(|other| other.name != palette.name),
                "duplicate palette {}",
                palette.name
            );
            assert!(
                palette
                    .shades
                    .windows(2)
                    .all(|pair| pair[0].color.lightness() > pair[1].color.lightness()),
                "lightness is not decreasing in palette {}",
                palette.name
            );
        }
    }

    #[test]
    fn builtin_colors_keep_their_values() {
        let blue = COLOR_PALETTES
            .iter()
            .find(|palette| palette.name == "blue")
            .unwrap();
        let red = COLOR_PALETTES
            .iter()
            .find(|palette| palette.name == "red")
            .unwrap();
        let neutral = COLOR_PALETTES
            .iter()
            .find(|palette| palette.name == "neutral")
            .unwrap();
        assert_eq!(
            blue.shades[6].color.to_string(),
            "oklch(0.546 0.245 262.881)"
        );
        assert_eq!(red.shades[6].color.to_string(), "oklch(0.577 0.245 27.325)");
        assert_eq!(neutral.shades[0].color.to_string(), "oklch(0.985 0 none)");
        assert_eq!(BLACK.to_string(), "oklch(0 0 0)");
        assert_eq!(WHITE.to_string(), "oklch(1 0 0)");
    }

    #[test]
    fn green_keeps_its_tailwind_values() {
        let green = COLOR_PALETTES
            .iter()
            .find(|palette| palette.name == "green")
            .unwrap();
        assert_eq!(
            green.shades[6].color.to_string(),
            "oklch(0.627 0.194 149.214)"
        );
        assert_eq!(
            green.shades[10].color.to_string(),
            "oklch(0.266 0.065 152.934)"
        );
    }
}

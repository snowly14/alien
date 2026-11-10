#![doc = env!("CARGO_PKG_DESCRIPTION")]

mod builtin;

pub use builtin::breakpoint::BREAKPOINTS;
pub use builtin::color::{BLACK, COLOR_PALETTES, ColorPalette, Shade, WHITE, color_shade};
pub use builtin::ease::EASINGS;
pub use builtin::font_size::FONT_SIZES;
pub use builtin::font_weight::FONT_WEIGHTS;
pub use builtin::radius::{RADIUS, Radius};
pub use builtin::space::SPACE;

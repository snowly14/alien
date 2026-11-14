/// A black shadow with pixel geometry and percentage opacity.
#[derive(Debug, Clone, Copy)]
pub struct Shadow {
    pub offset_y_px: u8,
    pub blur_px: u8,
    pub spread_px: i8,
    pub opacity_percent: u8,
}

/// Built-in elevation shadows.
pub const SHADOWS: &[(&str, Shadow)] = &[
    (
        "sm",
        Shadow {
            offset_y_px: 1,
            blur_px: 2,
            spread_px: 0,
            opacity_percent: 8,
        },
    ),
    (
        "md",
        Shadow {
            offset_y_px: 4,
            blur_px: 6,
            spread_px: -1,
            opacity_percent: 12,
        },
    ),
    (
        "lg",
        Shadow {
            offset_y_px: 10,
            blur_px: 15,
            spread_px: -3,
            opacity_percent: 16,
        },
    ),
];

/// Built-in cubic Bézier curves as [x1, y1, x2, y2].
pub const EASINGS: &[(&str, [f32; 4])] = &[
    ("in", [0.42, 0.0, 1.0, 1.0]),
    ("out", [0.0, 0.0, 0.58, 1.0]),
    ("inOut", [0.42, 0.0, 0.58, 1.0]),
];

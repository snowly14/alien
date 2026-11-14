use alien_tokens::{
    BLACK, BREAKPOINTS, COLOR_PALETTES, DURATIONS, EASINGS, FONT_SIZES, FONT_WEIGHTS, LINE_HEIGHTS,
    RADIUS, SHADOWS, SPACE, WHITE,
};

fn print_paths<Value>(family: &str, tokens: &[(&str, Value)]) {
    for (name, _) in tokens {
        println!("{family}.{name}");
    }
}

fn main() {
    print_paths("color", &[("black", BLACK), ("white", WHITE)]);
    for palette in COLOR_PALETTES {
        for shade in palette.shades {
            println!("color.{}.{}", palette.name, shade.step);
        }
    }

    print_paths("breakpoint", BREAKPOINTS);
    print_paths("duration", DURATIONS);
    print_paths("ease", EASINGS);
    print_paths("fontSize", FONT_SIZES);
    print_paths("fontWeight", FONT_WEIGHTS);
    print_paths("lineHeight", LINE_HEIGHTS);
    print_paths("radius", RADIUS);
    print_paths("shadow", SHADOWS);
    print_paths("space", SPACE);
}

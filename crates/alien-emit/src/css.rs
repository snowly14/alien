use std::fmt::Write;

use alien_color::Oklch;
use alien_ir::Condition;

/// Appends a background-color rule for a compiler-generated class name.
pub fn write_background_color(
    output: &mut String,
    class_name: &str,
    condition: Condition,
    color: Oklch,
) {
    let suffix = match condition {
        Condition::Base => "",
        Condition::Hover => ":hover",
    };

    // Writing these values to a String cannot fail.
    output.push('.');
    output.push_str(class_name);
    output.push_str(suffix);
    output.push_str("{background-color:");
    let _ = write!(output, "{color}}}");
}

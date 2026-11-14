use std::fmt::Write;

use alien_ir::Length;

pub(super) fn write_length(output: &mut String, length: &Length) {
    match length {
        Length::Literal { value, unit } => {
            let _ = write!(output, "{value}{}", unit.as_str());
        }
        Length::Sub { left, right } => {
            output.push_str("calc(");
            write_length(output, left);
            output.push_str(" - ");
            write_length(output, right);
            output.push(')');
        }
    }
}

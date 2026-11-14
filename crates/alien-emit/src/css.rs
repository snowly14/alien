mod declaration;
mod length;

use std::fmt::Write;

use alien_color::Oklch;
use alien_ir::{Property, PseudoClasses};
pub use declaration::{CssDeclaration, CssValue};

/// A resolved CSS condition.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CssCondition {
    pub min_width_rem: Option<f32>,
    pub pseudo_classes: PseudoClasses,
}

/// Appends a rule for a compiler-generated class name and a resolved declaration.
pub fn write_rule(
    output: &mut String,
    class_name: &str,
    condition: CssCondition,
    declaration: &CssDeclaration<'_>,
) {
    write_rules(output, class_name, condition, std::iter::once(declaration));
}

/// Appends declarations that share a selector and condition.
pub fn write_rules<'value: 'declaration, 'declaration>(
    output: &mut String,
    class_name: &str,
    condition: CssCondition,
    declarations: impl IntoIterator<Item = &'declaration CssDeclaration<'value>>,
) {
    let mut declarations = declarations.into_iter();
    let Some(first) = declarations.next() else {
        return;
    };

    if let Some(width) = condition.min_width_rem {
        let _ = write!(output, "@media(min-width:{width}rem){{");
    }
    output.push('.');
    output.push_str(class_name);
    for pseudo_class in condition.pseudo_classes.iter() {
        output.push_str(pseudo_class.as_css());
    }
    output.push('{');
    first.write_to(output);
    for declaration in declarations {
        output.push(';');
        declaration.write_to(output);
    }
    output.push('}');
    if condition.min_width_rem.is_some() {
        output.push('}');
    }
}

/// Appends a background-color rule for a compiler-generated class name.
pub fn write_background_color(
    output: &mut String,
    class_name: &str,
    condition: CssCondition,
    color: Oklch,
) {
    write_rule(
        output,
        class_name,
        condition,
        &CssDeclaration {
            property: Property::BackgroundColor,
            value: CssValue::Color(color, None),
            important: false,
        },
    );
}

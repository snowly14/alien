//! Experimental atomic stylesheet generation for complete style collections.
//!
//! The caller must isolate generated class names and resolve conflicting atoms
//! when composing styles. This lower-level API does not perform either task.

use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use alien_emit::{CssDeclaration, CssValue, write_rule};
use alien_ir::{GlobalConfig, Property};
use alien_parser::parse_styles;

use crate::{
    CompileError,
    class_name::compact_index,
    compile::{ResolvedStyle, resolve_styles},
};

/// An independently emitted declaration and its exact CSS condition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomicClass {
    /// CSS property of this declaration.
    pub property: String,
    /// Canonical identity of the pseudo-class and breakpoint condition.
    pub condition: String,
    /// Class selector, without the leading dot.
    pub class_name: String,
    /// Whether the declaration has `!important` priority.
    pub important: bool,
    /// First class index in the contiguous ordered alternatives, if any.
    pub variant_start: Option<usize>,
    /// Number of ordered alternatives.
    pub variant_count: usize,
    /// Overlap family whose variants must be ordered together.
    pub order_group: Option<&'static str>,
}

/// Atomic CSS and descriptors keyed by the source style name.
#[derive(Debug)]
pub struct AtomicStyles {
    pub css: String,
    pub styles: BTreeMap<String, Vec<AtomicClass>>,
}

const PADDING_PHYSICAL: [Property; 4] = [
    Property::PaddingTop,
    Property::PaddingRight,
    Property::PaddingBottom,
    Property::PaddingLeft,
];
const PADDING_LOGICAL: [Property; 2] = [Property::PaddingBlock, Property::PaddingInline];
const MARGIN_PHYSICAL: [Property; 4] = [
    Property::MarginTop,
    Property::MarginRight,
    Property::MarginBottom,
    Property::MarginLeft,
];
const MARGIN_LOGICAL: [Property; 2] = [Property::MarginBlock, Property::MarginInline];

fn group_size(
    all: &BTreeSet<Property>,
    shorthand: Property,
    physical: &[Property],
    logical: &[Property],
) -> usize {
    let physical_count = if all.contains(&shorthand) {
        physical.len()
    } else {
        physical
            .iter()
            .filter(|property| all.contains(property))
            .count()
    };
    let logical_count = logical
        .iter()
        .filter(|property| all.contains(property))
        .count();
    if physical_count > 0 && logical_count > 0 {
        physical_count + logical_count
    } else {
        0
    }
}

// These shorthands accept a single value in Alien's style language. Emitting
// independent longhands lets cx remove only the affected side of an earlier
// style. Logical/physical overlap and other shorthands need separate handling.
fn longhands(property: Property) -> &'static [Property] {
    use Property as P;
    match property {
        P::Padding => &[
            P::PaddingTop,
            P::PaddingRight,
            P::PaddingBottom,
            P::PaddingLeft,
        ],
        P::Margin => &[P::MarginTop, P::MarginRight, P::MarginBottom, P::MarginLeft],
        P::Gap => &[P::RowGap, P::ColumnGap],
        P::Inset => &[P::Top, P::Right, P::Bottom, P::Left],
        P::Overflow => &[P::OverflowX, P::OverflowY],
        P::BorderRadius => &[
            P::BorderTopLeftRadius,
            P::BorderTopRightRadius,
            P::BorderBottomRightRadius,
            P::BorderBottomLeftRadius,
        ],
        P::BorderWidth => &[
            P::BorderTopWidth,
            P::BorderRightWidth,
            P::BorderBottomWidth,
            P::BorderLeftWidth,
        ],
        P::BorderStyle => &[
            P::BorderTopStyle,
            P::BorderRightStyle,
            P::BorderBottomStyle,
            P::BorderLeftStyle,
        ],
        P::BorderColor => &[
            P::BorderTopColor,
            P::BorderRightColor,
            P::BorderBottomColor,
            P::BorderLeftColor,
        ],
        _ => &[],
    }
}

fn expand_declaration<'a>(
    declaration: &CssDeclaration<'a>,
    expand: bool,
) -> Vec<CssDeclaration<'a>> {
    use Property as P;
    if !expand {
        return vec![CssDeclaration {
            property: declaration.property,
            value: declaration.value.clone(),
            important: declaration.important,
        }];
    }
    if declaration.property == P::Flex {
        let values = match &declaration.value {
            CssValue::Number(grow) => Some([
                CssValue::Number(*grow),
                CssValue::Number(1.0),
                CssValue::Percent(0.0),
            ]),
            CssValue::Text("auto") => Some([
                CssValue::Number(1.0),
                CssValue::Number(1.0),
                CssValue::Text("auto"),
            ]),
            CssValue::Text("none") => Some([
                CssValue::Number(0.0),
                CssValue::Number(0.0),
                CssValue::Text("auto"),
            ]),
            _ => None,
        };
        if let Some(values) = values {
            return [P::FlexGrow, P::FlexShrink, P::FlexBasis]
                .into_iter()
                .zip(values)
                .map(|(property, value)| CssDeclaration {
                    property,
                    value,
                    important: declaration.important,
                })
                .collect();
        }
    }
    let expanded = longhands(declaration.property);
    let properties = if expanded.is_empty() {
        std::slice::from_ref(&declaration.property)
    } else {
        expanded
    };
    properties
        .iter()
        .map(|&property| CssDeclaration {
            property,
            value: declaration.value.clone(),
            important: declaration.important,
        })
        .collect()
}

/// Compiles a style module to a compact experimental atomic representation.
///
/// The generated short classes are unique only within this output. Descriptors
/// with overlapping shorthand and longhand properties require further handling
/// before they can be composed. This is not a replacement for `transform_project`.
///
/// # Errors
/// Returns an error when a style cannot be parsed or resolved.
pub fn compile_atomic_typescript(source: &str, path: &Path) -> Result<AtomicStyles, CompileError> {
    let parsed = parse_styles(source, path)
        .map_err(|diagnostics| crate::diagnostic::parse_error(source, path, diagnostics))?;
    let entries: Vec<_> = parsed
        .iter()
        .map(|style| (Cow::Borrowed(style.name.as_str()), style.rules.as_slice()))
        .collect();
    let config = GlobalConfig::default();
    let resolved = resolve_styles(&entries, "a", 0, &config)?;
    let prefix = crate::class_name::compact_project_prefix(&path.to_string_lossy());
    Ok(compile_resolved(&resolved, &prefix))
}

pub(crate) fn compile_resolved(resolved: &[ResolvedStyle<'_>], prefix: &str) -> AtomicStyles {
    let mut emitted_rules = Vec::new();
    let mut classes = BTreeMap::<String, (usize, String)>::new();
    let mut next_class = 0;
    let mut styles = BTreeMap::new();
    let all_properties: BTreeSet<_> = resolved
        .iter()
        .flat_map(|style| style.rules.iter().map(|rule| rule.declaration.property))
        .collect();
    let padding_slots = group_size(
        &all_properties,
        Property::Padding,
        &PADDING_PHYSICAL,
        &PADDING_LOGICAL,
    );
    let margin_slots = group_size(
        &all_properties,
        Property::Margin,
        &MARGIN_PHYSICAL,
        &MARGIN_LOGICAL,
    );
    for style in resolved {
        let mut atoms = Vec::with_capacity(style.rules.len());
        for rule in &style.rules {
            let property = rule.declaration.property;
            let expand = all_properties
                .iter()
                .any(|&other| other != property && property.conflicts_with(other));
            for declaration in expand_declaration(&rule.declaration, expand) {
                let property = declaration.property;
                let group = if padding_slots > 0
                    && (PADDING_PHYSICAL.contains(&property) || PADDING_LOGICAL.contains(&property))
                {
                    Some(("padding", padding_slots))
                } else if margin_slots > 0
                    && (MARGIN_PHYSICAL.contains(&property) || MARGIN_LOGICAL.contains(&property))
                {
                    Some(("margin", margin_slots))
                } else {
                    None
                };
                let mut signature = String::new();
                write_rule(&mut signature, "x", rule.condition, &declaration);
                let (start, class_name) = if let Some(value) = classes.get(&signature) {
                    value.clone()
                } else {
                    let start = next_class;
                    let first = format!("{prefix}{}", compact_index(start));
                    for priority in 0..group.map_or(1, |(_, slots)| slots) {
                        let class = format!("{prefix}{}", compact_index(next_class));
                        next_class += 1;
                        let mut css = String::new();
                        write_rule(&mut css, &class, rule.condition, &declaration);
                        emitted_rules.push((rule.condition, priority, css));
                    }
                    classes.insert(signature, (start, first.clone()));
                    (start, first)
                };
                let mut condition = String::new();
                if let Some(width) = rule.condition.min_width_rem {
                    use std::fmt::Write;
                    let _ = write!(condition, "@media(min-width:{width}rem)");
                }
                for pseudo in rule.condition.pseudo_classes.iter() {
                    condition.push_str(pseudo.as_css());
                }
                atoms.push(AtomicClass {
                    property: property.as_css().to_owned(),
                    condition,
                    class_name,
                    important: rule.declaration.important,
                    variant_start: group.map(|_| start),
                    variant_count: group.map_or(0, |(_, slots)| slots),
                    order_group: group.map(|(name, _)| name),
                });
            }
        }
        styles.insert(style.name.to_owned(), atoms);
    }
    // Fix condition precedence independently of which style first introduced a
    // shared class. Equal-specificity pseudo-classes can match simultaneously,
    // so first-encounter ordering would make unrelated source edits alter CSS.
    emitted_rules.sort_by(|(a, ap, _), (b, bp, _)| {
        a.pseudo_classes
            .specificity()
            .cmp(&b.pseudo_classes.specificity())
            .then_with(|| {
                a.min_width_rem
                    .unwrap_or_default()
                    .total_cmp(&b.min_width_rem.unwrap_or_default())
            })
            .then_with(|| a.pseudo_classes.cmp(&b.pseudo_classes))
            .then_with(|| ap.cmp(bp))
    });
    let css = emitted_rules.into_iter().map(|(_, _, css)| css).collect();
    AtomicStyles { css, styles }
}

#[cfg(test)]
mod tests {
    use super::compile_atomic_typescript;
    use std::path::Path;

    #[test]
    fn reuses_identical_declarations_and_distinguishes_conditions() {
        let source = r"import {alien} from 'alien'; const s = alien.create({
            red: {color:'red'}, blue: {color:'blue'}, redAgain: {color:'red'},
            focus: {on:{focus:{color:'red'}}}
        });";
        let output = compile_atomic_typescript(source, Path::new("test.ts")).unwrap();
        assert_eq!(
            output.styles["red"][0].class_name,
            output.styles["redAgain"][0].class_name
        );
        assert_ne!(
            output.styles["red"][0].class_name,
            output.styles["blue"][0].class_name
        );
        assert_ne!(
            output.styles["red"][0].class_name,
            output.styles["focus"][0].class_name
        );
        assert_eq!(output.styles["focus"][0].condition, ":focus");
        assert_eq!(output.css.matches("color:red").count(), 2);
    }

    #[test]
    fn condition_precedence_does_not_depend_on_first_occurrence() {
        let source = r"import {alien} from 'alien'; const s = alien.create({
            focus: {on:{focus:{color:'red'}}},
            hover: {on:{hover:{color:'blue'}}},
            base: {color:'green'}
        });";
        let output = compile_atomic_typescript(source, Path::new("test.ts")).unwrap();
        let base = output.css.find("color:green").unwrap();
        let hover = output.css.find(":hover").unwrap();
        let focus = output.css.find(":focus").unwrap();
        assert!(base < hover && hover < focus, "{}", output.css);
    }
}

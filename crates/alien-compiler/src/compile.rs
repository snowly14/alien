use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use alien_emit::{CssCondition, CssDeclaration, write_rule, write_rules};
use alien_ir::{GlobalConfig, PseudoClasses, Rule, Style};
use alien_parser::parse_styles;

use crate::{
    CompileError,
    resolve::{resolve_condition, resolve_declaration},
};

/// CSS and class names for one input module.
#[derive(Debug)]
pub struct CompiledStyles {
    pub css: String,
    pub classes: BTreeMap<String, String>,
}

/// Compiles one TypeScript or TSX module into CSS and class names.
///
/// # Errors
///
/// Returns an error when the source cannot be parsed or its styles cannot be compiled.
pub fn compile_typescript(source: &str, path: &Path) -> Result<CompiledStyles, CompileError> {
    let styles = parse_styles(source, path)
        .map_err(|diagnostics| crate::diagnostic::parse_error(source, path, diagnostics))?;
    compile_owned(styles, "a", 0, &GlobalConfig::default())
}

pub(crate) fn compile_owned(
    styles: Vec<Style>,
    prefix: &str,
    offset: usize,
    config: &GlobalConfig,
) -> Result<CompiledStyles, CompileError> {
    compile_entries(
        styles
            .into_iter()
            .map(|style| (Cow::Owned(style.name), style.rules)),
        prefix,
        offset,
        config,
    )
}

/// Compiles the intermediate styles from one module.
///
/// # Errors
///
/// Returns an error if a style name is repeated or a color shade cannot be resolved.
pub fn compile_styles(styles: &[Style]) -> Result<CompiledStyles, CompileError> {
    compile_entries(
        styles
            .iter()
            .map(|style| (Cow::Borrowed(style.name.as_str()), style.rules.as_slice())),
        "a",
        0,
        &GlobalConfig::default(),
    )
}

pub(crate) struct ResolvedRule<'a> {
    pub condition: CssCondition,
    pub declaration: CssDeclaration<'a>,
    /// Serialized with a fixed selector, for exact equality and byte costs.
    pub signature: String,
}

impl ResolvedRule<'_> {
    pub fn is_base(&self) -> bool {
        self.condition.min_width_rem.is_none() && self.condition.pseudo_classes.specificity() == 0
    }

    pub fn declaration_size(&self) -> usize {
        self.signature.len() - ".x{}".len()
    }
}

pub(crate) struct ResolvedStyle<'a> {
    pub name: &'a str,
    pub class: String,
    pub rules: Vec<ResolvedRule<'a>>,
}

fn compile_entries<'style, Rules: AsRef<[Rule]>>(
    styles: impl Iterator<Item = (Cow<'style, str>, Rules)>,
    prefix: &str,
    offset: usize,
    config: &GlobalConfig,
) -> Result<CompiledStyles, CompileError> {
    let styles: Vec<_> = styles.collect();
    let resolved = resolve_styles(&styles, prefix, offset, config)?;
    let grouped = compile_grouped(&resolved);
    Ok(crate::optimize::optimize(
        &resolved, prefix, offset, grouped,
    ))
}

pub(crate) fn resolve_styles<'a, R: AsRef<[Rule]>>(
    styles: &'a [(Cow<'_, str>, R)],
    prefix: &str,
    offset: usize,
    config: &'a GlobalConfig,
) -> Result<Vec<ResolvedStyle<'a>>, CompileError> {
    let mut names = BTreeSet::new();
    let mut resolved = Vec::with_capacity(styles.len());
    for (index, (name, rules)) in styles.iter().enumerate() {
        if !names.insert(name.as_ref()) {
            return Err(CompileError::DuplicateStyle(name.to_string()));
        }
        let mut rules = rules
            .as_ref()
            .iter()
            .map(|rule| {
                let condition = resolve_condition(&rule.condition)?;
                let declaration = resolve_declaration(&rule.declaration, config)?;
                let mut signature = String::new();
                write_rule(
                    &mut signature,
                    "x",
                    CssCondition {
                        min_width_rem: None,
                        pseudo_classes: PseudoClasses::default(),
                    },
                    &declaration,
                );
                Ok(ResolvedRule {
                    condition,
                    declaration,
                    signature,
                })
            })
            .collect::<Result<Vec<_>, CompileError>>()?;
        rules.sort_by(|left, right| {
            left.condition
                .pseudo_classes
                .specificity()
                .cmp(&right.condition.pseudo_classes.specificity())
                .then_with(|| {
                    left.condition
                        .min_width_rem
                        .unwrap_or_default()
                        .total_cmp(&right.condition.min_width_rem.unwrap_or_default())
                })
                .then_with(|| {
                    left.condition
                        .pseudo_classes
                        .cmp(&right.condition.pseudo_classes)
                })
        });
        resolved.push(ResolvedStyle {
            name,
            class: if prefix == "a" {
                format!("{prefix}{}", offset + index)
            } else {
                format!(
                    "{prefix}{}",
                    crate::class_name::compact_index(offset + index)
                )
            },
            rules,
        });
    }
    Ok(resolved)
}

pub(crate) fn compile_grouped(styles: &[ResolvedStyle<'_>]) -> CompiledStyles {
    let mut css = String::new();
    let mut classes = BTreeMap::new();
    // Reuse a whole style when nothing between its occurrences can change the
    // cascade for any of its properties. Record logical occurrences even when
    // no CSS is emitted, so later checks still see every intervening style.
    let mut previous: BTreeMap<String, (usize, &str)> = BTreeMap::new();
    let mut last_property: BTreeMap<alien_ir::Property, usize> = BTreeMap::new();
    for (index, style) in styles.iter().enumerate() {
        let mut key = String::new();
        for group in style.rules.chunk_by(|a, b| a.condition == b.condition) {
            write_rules(
                &mut key,
                "x",
                group[0].condition,
                group.iter().map(|rule| &rule.declaration),
            );
        }
        let reusable = previous.get(&key).copied().filter(|(last, _)| {
            style.rules.iter().all(|rule| {
                last_property.iter().all(|(property, seen)| {
                    *seen <= *last || !rule.declaration.property.conflicts_with(*property)
                })
            })
        });
        let class = if let Some((_, class)) = reusable {
            class
        } else {
            for group in style
                .rules
                .chunk_by(|left, right| left.condition == right.condition)
            {
                write_rules(
                    &mut css,
                    &style.class,
                    group[0].condition,
                    group.iter().map(|rule| &rule.declaration),
                );
            }
            &style.class
        };
        classes.insert(style.name.to_owned(), class.to_owned());
        previous.insert(key, (index, class));
        for rule in &style.rules {
            last_property.insert(rule.declaration.property, index);
        }
    }
    CompiledStyles { css, classes }
}

use std::borrow::Cow;
use std::collections::{BTreeMap, btree_map::Entry};
use std::path::Path;

use alien_color::Oklch;
use alien_emit::write_background_color;
use alien_ir::{ColorToken, Condition, Declaration, Rule, Style};
use alien_parser::parse_styles;
use alien_tokens::{BLACK, WHITE, color_shade};

use crate::CompileError;

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
    let styles = parse_styles(source, path).map_err(|diagnostics| CompileError::Parse {
        path: path.to_path_buf(),
        diagnostics,
    })?;
    compile_entries(
        styles
            .into_iter()
            .map(|style| (Cow::Owned(style.name), style.rules)),
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
    )
}

fn compile_entries<'style, Rules: AsRef<[Rule]>>(
    styles: impl Iterator<Item = (Cow<'style, str>, Rules)>,
) -> Result<CompiledStyles, CompileError> {
    let mut css = String::new();
    let mut classes = BTreeMap::new();

    for (index, (name, rules)) in styles.enumerate() {
        let entry = match classes.entry(name.into_owned()) {
            Entry::Vacant(entry) => entry,
            Entry::Occupied(entry) => {
                return Err(CompileError::DuplicateStyle(entry.key().clone()));
            }
        };

        let class_name = format!("a{index}");
        let rules = rules.as_ref();
        for condition in [Condition::Base, Condition::Hover] {
            for rule in rules.iter().filter(|rule| rule.condition == condition) {
                match &rule.declaration {
                    Declaration::BackgroundColor(token) => {
                        let color = resolve_color(token)?;
                        write_background_color(&mut css, &class_name, condition, color);
                    }
                }
            }
        }

        entry.insert(class_name);
    }

    Ok(CompiledStyles { css, classes })
}

fn resolve_color(token: &ColorToken) -> Result<Oklch, CompileError> {
    match token {
        ColorToken::Black => Ok(BLACK),
        ColorToken::White => Ok(WHITE),
        ColorToken::Shade { palette, step } => {
            color_shade(palette, *step).ok_or_else(|| CompileError::UnknownColorShade {
                palette: palette.clone(),
                step: *step,
            })
        }
    }
}

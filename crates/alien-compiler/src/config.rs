use std::path::Path;

use alien_ir::GlobalConfig;
use alien_tokens::{
    BREAKPOINTS, COLOR_PALETTES, DURATIONS, EASINGS, FONT_SIZES, FONT_WEIGHTS, LINE_HEIGHTS,
    RADIUS, SHADOWS, SPACE,
};

use crate::{CompileError, resolve::resolve_value};

/// Parses and validates project configuration without executing TypeScript.
///
/// # Errors
/// Returns errors for invalid configuration, unresolved references, or attempts to replace built-ins.
pub fn compile_globals(source: &str, path: &Path) -> Result<GlobalConfig, CompileError> {
    let config = alien_parser::parse_globals(source, path)
        .map_err(|diagnostics| crate::diagnostic::parse_error(source, path, diagnostics))?;
    for (key, value) in &config.tokens {
        let (family, name) = key
            .split_once('.')
            .ok_or_else(|| CompileError::Configuration(format!("invalid token path `{key}`")))?;
        let builtin = match family {
            "color" => {
                matches!(name, "black" | "white")
                    || COLOR_PALETTES.iter().any(|palette| palette.name == name)
            }
            "space" => contains(SPACE, name),
            "radius" => contains(RADIUS, name),
            "fontSize" => contains(FONT_SIZES, name),
            "fontWeight" => contains(FONT_WEIGHTS, name),
            "lineHeight" => contains(LINE_HEIGHTS, name),
            "duration" => contains(DURATIONS, name),
            "ease" => contains(EASINGS, name),
            "shadow" => contains(SHADOWS, name),
            "breakpoint" => contains(BREAKPOINTS, name),
            _ => false,
        };
        if builtin {
            return Err(CompileError::Configuration(format!(
                "{}: `{key}` is built-in; choose a new semantic token name",
                path.display()
            )));
        }
        resolve_value(value, &config, 0)?;
    }
    Ok(config)
}

fn contains<T>(table: &[(&str, T)], name: &str) -> bool {
    table.iter().any(|(key, _)| *key == name)
}

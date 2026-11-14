use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    fmt::Write,
    ops::Range,
    path::Path,
};

use alien_emit::write_class_map;
use alien_ir::{GlobalConfig, Style};
use alien_parser::parse_style_blocks_with_mode;

use crate::{
    CompileError, CompiledStyles,
    class_name::class_prefix,
    compile::{ResolvedStyle, compile_grouped, resolve_styles},
};

/// Source with Alien calls replaced by class maps, plus the generated CSS.
#[derive(Debug)]
pub struct TransformedModule {
    pub code: String,
    pub css: String,
    pub edits: Vec<SourceEdit>,
}

/// A replacement with UTF-16 offsets, compatible with JavaScript source-map tools.
#[derive(Debug)]
pub struct SourceEdit {
    pub start: usize,
    pub end: usize,
    pub replacement: String,
}

/// Transforms one source module with a stable, project-relative module identifier.
///
/// Returns `None` for modules without an Alien style call.
///
/// # Errors
///
/// Returns an error if a style cannot be extracted or compiled.
pub fn transform_typescript(
    source: &str,
    path: &Path,
    module_id: &str,
) -> Result<Option<TransformedModule>, CompileError> {
    transform_with_config(source, path, module_id, &GlobalConfig::default())
}

/// Transforms a module with the project's token registry and policy.
///
/// # Errors
/// Returns extraction, policy, or token resolution errors.
pub fn transform_with_config(
    source: &str,
    path: &Path,
    module_id: &str,
    config: &GlobalConfig,
) -> Result<Option<TransformedModule>, CompileError> {
    let prepared = prepare(source, path, config, "")?;
    if prepared.calls.is_empty() {
        return Ok(None);
    }
    let prefix = class_prefix(module_id);
    let entries = borrowed_entries(&prepared);
    let resolved = resolve_styles(&entries, &prefix, 0, config)?;
    let output = compile_module(&resolved, &prepared, &prefix, 0);
    Ok(Some(rewrite(
        source,
        &prepared,
        &output.classes,
        output.css,
    )))
}

struct Call {
    span: Range<usize>,
    keys: Vec<(String, String)>,
}

struct PreparedModule {
    entries: Vec<Style>,
    calls: Vec<Call>,
}

fn borrowed_entries(prepared: &PreparedModule) -> Vec<(Cow<'_, str>, &[alien_ir::Rule])> {
    prepared
        .entries
        .iter()
        .map(|style| (Cow::Borrowed(style.name.as_str()), style.rules.as_slice()))
        .collect()
}

fn prepare(
    source: &str,
    path: &Path,
    config: &GlobalConfig,
    key_prefix: &str,
) -> Result<PreparedModule, CompileError> {
    let blocks = parse_style_blocks_with_mode(source, path, config.mode)
        .map_err(|diagnostics| crate::diagnostic::parse_error(source, path, diagnostics))?;
    let mut entries = Vec::new();
    let mut calls = Vec::with_capacity(blocks.len());
    let single_call = blocks.len() == 1;
    for block in blocks {
        let mut names = BTreeSet::new();
        let mut keys = Vec::with_capacity(block.styles.len());
        for mut style in block.styles {
            if !names.insert(style.name.clone()) {
                return Err(CompileError::DuplicateStyle(style.name));
            }
            let key = if single_call && key_prefix.is_empty() {
                style.name.clone()
            } else {
                format!("{key_prefix}{}", entries.len())
            };
            keys.push((std::mem::replace(&mut style.name, key.clone()), key));
            entries.push(style);
        }
        calls.push(Call {
            span: block.span,
            keys,
        });
    }
    Ok(PreparedModule { entries, calls })
}

// Keep per-call optimization as a fallback. A conflicting declaration in a
// later call must not disable otherwise profitable sharing inside an earlier one.
fn compile_module(
    styles: &[ResolvedStyle<'_>],
    prepared: &PreparedModule,
    prefix: &str,
    offset: usize,
) -> CompiledStyles {
    if prepared.calls.len() == 1 {
        return crate::optimize::optimize(styles, prefix, offset, compile_grouped(styles));
    }
    let mut baseline = CompiledStyles {
        css: String::new(),
        classes: BTreeMap::new(),
    };
    let mut start = 0;
    for call in &prepared.calls {
        let end = start + call.keys.len();
        let group = &styles[start..end];
        let output =
            crate::optimize::optimize(group, prefix, offset + start, compile_grouped(group));
        baseline.css.push_str(&output.css);
        baseline.classes.extend(output.classes);
        start = end;
    }
    crate::optimize::optimize(styles, &format!("{prefix}m"), offset, baseline)
}

fn rewrite(
    source: &str,
    prepared: &PreparedModule,
    classes: &BTreeMap<String, String>,
    css: String,
) -> TransformedModule {
    let mut code = String::with_capacity(source.len());
    let mut cursor = 0;
    let mut utf16_cursor = 0;
    let mut edits = Vec::with_capacity(prepared.calls.len());
    for Call { span, keys } in &prepared.calls {
        let classes: BTreeMap<_, _> = keys
            .iter()
            .map(|(name, key)| (name.clone(), classes[key].clone()))
            .collect();
        let unchanged = &source[cursor..span.start];
        let start = utf16_cursor + unchanged.encode_utf16().count();
        let end = start + source[span.clone()].encode_utf16().count();
        let mut replacement = String::new();
        write_class_map(&mut replacement, &classes);
        code.push_str(unchanged);
        code.push_str(&replacement);
        edits.push(SourceEdit {
            start,
            end,
            replacement,
        });
        utf16_cursor = end;
        cursor = span.end;
    }
    code.push_str(&source[cursor..]);
    TransformedModule { code, css, edits }
}

/// One style module in an ordered, complete project compilation.
pub struct ProjectSource<'a> {
    pub source: &'a str,
    pub path: &'a Path,
    pub module_id: &'a str,
}

/// Modules and their single shared stylesheet. Individual module CSS is empty.
pub struct TransformedProject {
    pub modules: Vec<TransformedModule>,
    pub css: String,
}

/// Optimizes a complete, ordered set of style modules, retaining cascade order.
/// All returned modules must be deployed together with the returned stylesheet.
///
/// # Errors
/// Returns diagnostics for invalid styles, duplicate module identities or sources without style calls.
pub fn transform_project(
    sources: &[ProjectSource<'_>],
    config: &GlobalConfig,
) -> Result<TransformedProject, CompileError> {
    let mut identities = BTreeSet::new();
    let mut identity = String::from("alien-project-sharing:");
    let mut prepared = Vec::with_capacity(sources.len());
    for (index, input) in sources.iter().enumerate() {
        if !identities.insert(input.module_id) {
            return Err(CompileError::Configuration(format!(
                "duplicate module identity: {}",
                input.module_id
            )));
        }
        let _ = write!(identity, "{}:{}", input.module_id.len(), input.module_id);
        let module = prepare(input.source, input.path, config, &format!("{index}:"))?;
        if module.calls.is_empty() {
            return Err(CompileError::Configuration(format!(
                "{}: a project style module must contain an alien.create() declaration",
                input.path.display()
            )));
        }
        prepared.push(module);
    }
    let entries: Vec<_> = prepared.iter().map(borrowed_entries).collect();
    let mut resolved = Vec::new();
    let mut baseline = CompiledStyles {
        css: String::new(),
        classes: BTreeMap::new(),
    };
    let prefix = class_prefix(&identity);
    let mut offset = 0;
    for (entries, module) in entries.iter().zip(&prepared) {
        let styles = resolve_styles(entries, &prefix, offset, config)?;
        let output = compile_module(&styles, module, &prefix, offset);
        offset += entries.len();
        baseline.css.push_str(&output.css);
        baseline.classes.extend(output.classes);
        resolved.extend(styles);
    }
    // Reserve a separate sharing offset so local and project candidates cannot collide.
    let output = crate::optimize::optimize(&resolved, &prefix, offset, baseline);
    let modules = sources
        .iter()
        .zip(&prepared)
        .map(|(input, module)| rewrite(input.source, module, &output.classes, String::new()))
        .collect();
    Ok(TransformedProject {
        modules,
        css: output.css,
    })
}

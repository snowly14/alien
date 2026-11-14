#![doc = env!("CARGO_PKG_DESCRIPTION")]

use alien_compiler::{
    ProjectSource, TransformedModule, compile_globals, transform_project, transform_typescript,
    transform_with_config,
};
use alien_ir::{GlobalConfig, Mode, Reset};
use napi::bindgen_prelude::{Error, Result};
use napi_derive::napi;
use std::path::Path;

#[napi(object)]
pub struct SourceEdit {
    pub start: u32,
    pub end: u32,
    pub replacement: String,
}

#[napi(object)]
pub struct TransformResult {
    pub code: String,
    pub css: String,
    pub edits: Vec<SourceEdit>,
}

#[napi(object)]
pub struct ProjectInput {
    pub source: String,
    pub path: String,
    pub module_id: String,
}

#[napi(object)]
pub struct ProjectResult {
    pub modules: Vec<TransformResult>,
    pub css: String,
}

fn convert(result: Option<TransformedModule>) -> Result<Option<TransformResult>> {
    result
        .map(|result| {
            let edits = result
                .edits
                .into_iter()
                .map(|edit| {
                    Ok(SourceEdit {
                        start: u32::try_from(edit.start)
                            .map_err(|_| Error::from_reason("source file is too large"))?,
                        end: u32::try_from(edit.end)
                            .map_err(|_| Error::from_reason("source file is too large"))?,
                        replacement: edit.replacement,
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(TransformResult {
                code: result.code,
                css: result.css,
                edits,
            })
        })
        .transpose()
}

#[napi]
pub fn transform(
    source: String,
    path: String,
    module_id: String,
) -> Result<Option<TransformResult>> {
    convert(
        transform_typescript(&source, Path::new(&path), &module_id)
            .map_err(|error| Error::from_reason(error.to_string()))?,
    )
}

/// One immutable project configuration, reused across all transformed modules.
#[napi]
pub struct Compiler {
    config: GlobalConfig,
}

#[napi]
impl Compiler {
    #[napi(constructor)]
    pub fn new(source: Option<String>, path: Option<String>) -> Result<Self> {
        let config = source
            .map(|source| {
                compile_globals(
                    &source,
                    Path::new(path.as_deref().unwrap_or("alien.globals.ts")),
                )
            })
            .transpose()
            .map_err(|error| Error::from_reason(error.to_string()))?
            .unwrap_or_default();
        Ok(Self { config })
    }

    #[napi]
    pub fn transform(
        &self,
        source: String,
        path: String,
        module_id: String,
    ) -> Result<Option<TransformResult>> {
        convert(
            transform_with_config(&source, Path::new(&path), &module_id, &self.config)
                .map_err(|error| Error::from_reason(error.to_string()))?,
        )
    }

    #[napi]
    pub fn transform_project(&self, sources: Vec<ProjectInput>) -> Result<ProjectResult> {
        let inputs: Vec<_> = sources
            .iter()
            .map(|input| ProjectSource {
                source: &input.source,
                path: Path::new(&input.path),
                module_id: &input.module_id,
            })
            .collect();
        let output = transform_project(&inputs, &self.config)
            .map_err(|error| Error::from_reason(error.to_string()))?;
        let modules = output
            .modules
            .into_iter()
            .map(|module| {
                convert(Some(module))?
                    .ok_or_else(|| Error::from_reason("missing compiled project module"))
            })
            .collect::<Result<_>>()?;
        Ok(ProjectResult {
            modules,
            css: output.css,
        })
    }

    #[napi(getter)]
    pub fn mode(&self) -> String {
        match self.config.mode {
            Mode::Free => "free",
            Mode::Strict => "strict",
        }
        .to_owned()
    }

    #[napi(getter)]
    pub fn reset(&self) -> Option<String> {
        let level = match self.config.reset {
            Reset::Disabled => return None,
            Reset::Light => "light",
            Reset::Medium => "medium",
            Reset::Strong => "strong",
        };
        Some(level.to_owned())
    }

    #[napi(getter)]
    pub fn token_paths(&self) -> Vec<String> {
        self.config.tokens.keys().cloned().collect()
    }
}

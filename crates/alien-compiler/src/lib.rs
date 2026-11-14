#![doc = env!("CARGO_PKG_DESCRIPTION")]

mod class_name;
mod compile;
mod error;
mod resolve;
mod transform;

pub use compile::{CompiledStyles, compile_styles, compile_typescript};
pub use error::CompileError;
pub use transform::{TransformedModule, transform_typescript};

#[cfg(test)]
mod tests;

mod config;
pub use config::compile_globals;
pub use transform::{SourceEdit, transform_with_config};

mod diagnostic;

mod optimize;

pub use transform::{ProjectSource, TransformedProject, transform_project};

#![doc = env!("CARGO_PKG_DESCRIPTION")]

mod compile;
mod error;

pub use compile::{CompiledStyles, compile_styles, compile_typescript};
pub use error::CompileError;

#[cfg(test)]
mod tests;

use std::fmt;
use std::path::PathBuf;

/// An error encountered while compiling a style.
#[derive(Debug, PartialEq, Eq)]
pub enum CompileError {
    Parse {
        path: PathBuf,
        diagnostics: Vec<String>,
    },
    DuplicateStyle(String),
    UnknownColorShade {
        palette: String,
        step: u16,
    },
}

impl fmt::Display for CompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse { path, diagnostics } => {
                write!(formatter, "{}: {}", path.display(), diagnostics.join("\n"))
            }
            Self::DuplicateStyle(name) => write!(formatter, "duplicate style name: {name}"),
            Self::UnknownColorShade { palette, step } => {
                write!(formatter, "unknown color token: color.{palette}.{step}")
            }
        }
    }
}

impl std::error::Error for CompileError {}

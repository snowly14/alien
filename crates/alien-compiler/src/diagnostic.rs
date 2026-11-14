use std::path::Path;

use crate::CompileError;

pub(crate) fn parse_error(source: &str, path: &Path, diagnostics: Vec<String>) -> CompileError {
    CompileError::Parse {
        path: path.to_path_buf(),
        diagnostics: diagnostics
            .into_iter()
            .map(|message| render(source, &message))
            .collect(),
    }
}

fn render(source: &str, message: &str) -> String {
    let Some((_, range)) = message.rsplit_once("(bytes ") else {
        return message.to_owned();
    };
    let Some(start) = range
        .split_once("..")
        .and_then(|(start, _)| start.parse::<usize>().ok())
        .filter(|start| source.is_char_boundary(*start))
    else {
        return message.to_owned();
    };
    let prefix = &source[..start];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let line_start = prefix.rfind('\n').map_or(0, |index| index + 1);
    let column = source[line_start..start].chars().count() + 1;
    let text = source[line_start..].lines().next().unwrap_or_default();
    format!(
        "{message}\n  --> line {line}, column {column}\n  | {text}\n  | {}^",
        " ".repeat(column - 1)
    )
}

#[cfg(test)]
mod tests {
    use super::render;

    #[test]
    fn renders_unicode_locations_without_treating_bytes_as_columns() {
        let source = "const title = '👽';\n  invalid";
        let start = source.find("invalid").unwrap();
        let message = render(
            source,
            &format!("unsupported value (bytes {start}..{})", start + 7),
        );
        assert!(message.contains("line 2, column 3"));
        assert!(message.contains("  |   invalid"));
    }
}

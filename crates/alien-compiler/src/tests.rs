use std::path::Path;

use crate::{CompileError, compile_typescript};

#[test]
fn compiles_tsx_with_aliased_alien_imports() {
    let source = r#"
        import { alien as ui, tokens as theme } from "alien";
        export const styles = ui.create({
            button: {
                on: { hover: { backgroundColor: ui.color.green[700] } },
                backgroundColor: theme.color.green[600],
            },
        });
        export const button = <button className={styles.button}>Save</button>;
    "#;
    let result = compile_typescript(source, Path::new("button.tsx")).unwrap();

    assert_eq!(result.classes["button"], "a0");
    assert_eq!(
        result.css,
        concat!(
            ".a0{background-color:oklch(0.627 0.194 149.214)}",
            ".a0:hover{background-color:oklch(0.527 0.154 150.069)}",
        )
    );
}

#[test]
fn surfaces_source_errors_through_the_compiler() {
    let path = Path::new("broken.ts");
    let error = compile_typescript("const styles = {", path).unwrap_err();

    let CompileError::Parse {
        path: file,
        diagnostics,
    } = error
    else {
        panic!("expected a parser error");
    };
    assert_eq!(file, path);
    assert_ne!(diagnostics, Vec::<String>::new());
}

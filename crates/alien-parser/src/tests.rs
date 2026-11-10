use std::path::Path;

use alien_ir::{ColorToken, Condition, Declaration};

use crate::{extract::Expr, extract_styles, parse_styles};

#[test]
fn extracts_the_button_fixture_into_ir() {
    let source = include_str!("../fixtures/button.ts");
    let styles = parse_styles(source, Path::new("button.ts")).unwrap();
    let [button] = styles.as_slice() else {
        panic!("expected one button style");
    };

    assert_eq!(button.name, "button");
    assert_eq!(button.rules.len(), 2);
    for (rule, (expected_condition, expected_step)) in button
        .rules
        .iter()
        .zip([(Condition::Base, 600), (Condition::Hover, 700)])
    {
        assert_eq!(rule.condition, expected_condition);
        let Declaration::BackgroundColor(ColorToken::Shade { palette, step }) = &rule.declaration
        else {
            panic!("expected a palette shade");
        };
        assert_eq!(palette, "green");
        assert_eq!(*step, expected_step);
    }
}

#[test]
fn ignores_unrelated_and_shadowed_create_calls() {
    let source = r#"
        import { alien, tokens } from "alien";
        import { alien as foreign } from "another-package";
        const ignored = tokens.create({ fake: {} });
        const unrelated = foreign.create({ fake: {} });
        function local(alien: { create: (styles: object) => unknown }) {
            const ignored = alien.create({ fake: {} });
            return ignored;
        }
    "#;

    assert!(
        parse_styles(source, Path::new("unrelated.ts"))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn rejects_an_index_that_would_be_truncated() {
    let source = r#"
        import { alien, tokens } from "alien";
        const styles = alien.create({ button: { backgroundColor: tokens.color.green[600.5] } });
    "#;
    let errors = parse_styles(source, Path::new("index.ts")).unwrap_err();

    assert!(errors[0].contains("token index must be a non-negative integer"));
    assert!(errors[0].contains("bytes"));
}

#[test]
fn rejects_a_color_from_an_unrelated_binding() {
    let source = r#"
        import { alien } from "alien";
        import { tokens } from "another-package";
        const styles = alien.create({ button: { backgroundColor: tokens.color.green[600] } });
    "#;
    let errors = parse_styles(source, Path::new("tokens.ts")).unwrap_err();

    assert!(errors[0].contains("expects a color token imported from `alien`"));
}

#[test]
fn raw_extraction_keeps_values_beyond_the_compiler_subset() {
    let source = r#"
        import { alien as ui } from "alien";
        const styles = ui.create({ caption: { opacity: 0.5, content: "hello" } });
    "#;
    let expressions = extract_styles(source).unwrap();
    let [Expr::Object(styles)] = expressions.as_slice() else {
        panic!("expected one extracted object");
    };
    let [(name, Expr::Object(properties))] = styles.as_slice() else {
        panic!("expected a single named style");
    };
    assert_eq!(name, "caption");
    assert!(matches!(
        properties.as_slice(),
        [(opacity, Expr::Number(0.5)), (content, Expr::Str(value))]
            if opacity == "opacity" && content == "content" && value == "hello"
    ));
}

#[test]
fn reports_unsupported_style_properties_with_a_source_range() {
    let source = r#"
        import { alien } from "alien";
        const styles = alien.create({ caption: { opacity: 0.5 } });
    "#;
    let errors = parse_styles(source, Path::new("caption.ts")).unwrap_err();

    assert!(errors[0].contains("style `caption`: unsupported style property `opacity`"));
    assert!(errors[0].contains("bytes"));
}

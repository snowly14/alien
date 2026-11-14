use std::path::Path;

use crate::{CompileError, compile_typescript, transform_typescript};

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

#[test]
fn rewrites_each_call_and_keeps_unicode_source_intact() {
    let path = Path::new("buttons.ts");
    let source = r#"
        import { alien, tokens } from "alien";
        const title = "Été 🟢";
        export const first = alien.create({
            button: { backgroundColor: tokens.color.green[600] },
        });
        export const second = alien.create({
            button: { backgroundColor: tokens.color.blue[600] },
        });
    "#;
    let result = transform_typescript(source, path, "buttons.ts")
        .unwrap()
        .unwrap();
    let classes: Vec<_> = result
        .css
        .split('}')
        .filter_map(|rule| rule.split_once('{'))
        .collect();
    let [(first, _), (second, _)] = classes.as_slice() else {
        panic!("expected one rule per call");
    };

    assert_ne!(first, second);
    assert!(result.code.contains(r#"const title = "Été 🟢";"#));
    assert!(result.code.contains(&format!(
        "export const first = ({{\"button\":\"{}\"}})",
        &first[1..]
    )));
    assert!(result.code.contains(&format!(
        "export const second = ({{\"button\":\"{}\"}})",
        &second[1..]
    )));

    let other = transform_typescript(source, path, "other/buttons.ts")
        .unwrap()
        .unwrap();
    assert_ne!(result.css, other.css);
}

#[test]
fn leaves_unrelated_modules_out_of_the_transform() {
    let result = transform_typescript("export const value = 1;", Path::new("value.ts"), "value.ts");

    assert!(result.unwrap().is_none());
}

#[test]
fn compiles_padding_and_independent_border_fields() {
    let source = r#"
        import { alien, css as keywords, tokens } from "alien";
        import { calc, px as pixels, rem } from "alien/values";
        const styles = alien.create({
            button: {
                padding: tokens.space.md,
                borderRadius: tokens.radius.lg,
                border: {
                    width: calc.sub(rem(1), pixels(2)),
                    style: keywords.borderStyle.solid,
                    color: tokens.color.black,
                },
                on: { hover: { border: { color: tokens.color.white } } },
            },
        });
    "#;
    let result = compile_typescript(source, Path::new("button.ts")).unwrap();

    assert_eq!(
        result.css,
        concat!(
            ".a0{padding:1rem;border-radius:0.5rem;border-width:calc(1rem - 2px);",
            "border-style:solid;border-color:oklch(0 0 0)}",
            ".a0:hover{border-color:oklch(1 0 0)}",
        )
    );
}

#[test]
fn compiles_logical_padding_and_border_color_keywords() {
    let source = r#"
        import { alien, css, tokens } from "alien";
        const styles = alien.create({
            button: {
                paddingBlock: tokens.space.sm,
                paddingInline: tokens.space.lg,
                border: { color: css.color.transparent },
                on: { hover: { border: { color: css.color.currentColor } } },
            },
        });
    "#;
    let result = compile_typescript(source, Path::new("button.ts")).unwrap();

    assert_eq!(
        result.css,
        concat!(
            ".a0{padding-block:0.5rem;padding-inline:1.5rem;border-color:transparent}",
            ".a0:hover{border-color:currentColor}",
        )
    );
}

#[test]
fn rejects_overlapping_padding_in_the_same_condition() {
    let source = r#"
        import { alien, tokens } from "alien";
        const styles = alien.create({
            button: {
                padding: tokens.space.md,
                paddingInline: tokens.space.lg,
            },
        });
    "#;
    let error = compile_typescript(source, Path::new("button.ts")).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("overlapping paddingInline declaration")
    );
}

#[test]
fn rejects_duplicate_border_fields() {
    let source = r#"
        import { alien, css } from "alien";
        const styles = alien.create({
            button: {
                border: { style: css.borderStyle.solid, style: css.borderStyle.dashed },
            },
        });
    "#;
    let error = compile_typescript(source, Path::new("border.ts")).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("duplicate border.style declaration")
    );
}

#[test]
fn compiles_builtin_token_families_in_styles_and_responsive_motion() {
    let source = r#"
        import { alien, css, custom, tokens as t } from "alien";
        import { ms, px } from "alien/values";

        export const styles = alien.create({
            card: {
                on: {
                    breakpoint: {
                        lg: { gap: t.space.xl },
                        md: {
                            gap: t.space.md,
                            on: { hover: { gap: t.space.lg } },
                        },
                    },
                    hover: { gap: t.space.sm },
                },
                paddingTop: t.space.xxs,
                marginBlock: t.space.none,
                marginInline: css.margin.auto,
                rowGap: t.space.xs,
                columnGap: t.space.sm,
                borderTopLeftRadius: t.radius.lg,
                borderTop: { color: t.color.blue[600] },
                outlineColor: t.color.black,
                fontSize: t.fontSize.base,
                fontWeight: t.fontWeight.semibold,
                lineHeight: t.lineHeight.relaxed,
                boxShadow: t.shadow.md,
                transition: {
                    property: css.transitionProperty.boxShadow,
                    duration: t.duration.fast,
                    delay: ms(-25),
                    ease: t.ease.out,
                },
                animation: {
                    name: custom("fade-in"),
                    duration: t.duration.slow,
                    delay: t.duration.normal,
                    ease: t.ease.inOut,
                },
            },
            label: {
                fontSize: px(14),
                fontWeight: t.fontWeight.medium,
                lineHeight: 1.5,
            },
        });
    "#;
    let result = compile_typescript(source, Path::new("card.ts")).unwrap();

    assert_eq!(
        result.css,
        concat!(
            ".a0{padding-top:0.125rem;margin-block:0;margin-inline:auto;row-gap:0.25rem;",
            "column-gap:0.5rem;border-top-left-radius:0.5rem;border-top-color:oklch(0.546 0.245 262.881);",
            "outline-color:oklch(0 0 0);font-size:1rem;font-weight:600;line-height:1.625;",
            "box-shadow:0 4px 6px -1px rgb(0 0 0/12%);",
            "transition-property:box-shadow;transition-duration:120ms;transition-delay:-25ms;",
            "transition-timing-function:cubic-bezier(0,0,0.58,1);",
            "animation-name:fade-in;animation-duration:300ms;animation-delay:180ms;",
            "animation-timing-function:cubic-bezier(0.42,0,0.58,1)}",
            "@media(min-width:48rem){.a0{gap:1rem}}",
            "@media(min-width:64rem){.a0{gap:2rem}}",
            ".a0:hover{gap:0.5rem}",
            "@media(min-width:48rem){.a0:hover{gap:1.5rem}}",
            ".a1{font-size:14px;font-weight:500;line-height:1.5}",
        )
    );
}

#[test]
fn rejects_unknown_builtin_tokens_and_incompatible_families() {
    let template = r#"
        import { alien, tokens } from "alien";
        const styles = alien.create({ card: { DECLARATION } });
    "#;
    for (declaration, diagnostic) in [
        (
            "boxShadow: tokens.shadow.missing",
            "unknown token: shadow.missing",
        ),
        (
            "lineHeight: tokens.space.sm",
            "expected a `lineHeight` token",
        ),
        (
            "on: { breakpoint: { tablet: { gap: tokens.space.sm } } }",
            "unknown token: breakpoint.tablet",
        ),
        (
            "transition: { duration: tokens.ease.out }",
            "expected a `duration` token",
        ),
    ] {
        let source = template.replace("DECLARATION", declaration);
        let error = compile_typescript(&source, Path::new("card.ts")).unwrap_err();
        assert!(error.to_string().contains(diagnostic), "{error}");
    }
}

#[test]
fn compiles_cursor_keywords_and_rejects_unknown_values() {
    let source = r#"
        import { alien, css as keywords } from "alien";
        const styles = alien.create({
            button: {
                cursor: keywords.cursor.pointer,
                on: { hover: { cursor: keywords.cursor.notAllowed } },
            },
        });
    "#;
    let path = Path::new("button.ts");
    let result = compile_typescript(source, path).unwrap();
    assert_eq!(
        result.css,
        ".a0{cursor:pointer}.a0:hover{cursor:not-allowed}"
    );

    for invalid in [
        "keywords.cursor.unknown",
        "keywords.display.grid",
        "\"notAllowed\"",
    ] {
        let error = compile_typescript(&source.replace("keywords.cursor.pointer", invalid), path)
            .unwrap_err();
        assert!(error.to_string().contains("expected a `cursor` keyword"));
    }
}

#[test]
fn compiles_layout_typography_and_appearance_values() {
    let source = r#"
        import { alien, css, custom, tokens } from "alien";
        import { fr, important, percent, px, rem } from "alien/values";

        const styles = alien.create({
            panel: {
                display: css.display.grid,
                gridTemplateColumns: [css.gridTrack.minContent, fr(1), rem(2)],
                gridAutoFlow: css.gridAutoFlow.rowDense,
                overflowX: css.overflowX.hidden,
                width: percent(80),
                left: px(-2),
                opacity: percent(80),
                fontFamily: [custom("Inter \"UI\""), css.fontFamily.systemUi],
                letterSpacing: rem(-0.025),
                textDecorationLine: css.textDecorationLine.lineThrough,
                backgroundRepeat: css.backgroundRepeat.noRepeat,
                outlineWidth: css.outlineWidth.thick,
                outlineStyle: css.outlineStyle.auto,
                outlineOffset: px(-1),
                rowGap: css.rowGap.normal,
                columnGap: css.columnGap.normal,
                border: important({
                    width: px(1),
                    color: important(tokens.color.black),
                }),
                pointerEvents: important(css.pointerEvents.none),
            },
        });
    "#;
    let result = compile_typescript(source, Path::new("panel.ts")).unwrap();
    assert_eq!(
        result.css,
        concat!(
            ".a0{display:grid;grid-template-columns:min-content 1fr 2rem;grid-auto-flow:row dense;",
            "overflow-x:hidden;width:80%;left:-2px;opacity:80%;font-family:\"Inter \\\"UI\\\"\",system-ui;",
            "letter-spacing:-0.025rem;text-decoration-line:line-through;background-repeat:no-repeat;",
            "outline-width:thick;outline-style:auto;outline-offset:-1px;row-gap:normal;column-gap:normal;",
            "border-width:1px!important;border-color:oklch(0 0 0)!important;pointer-events:none!important}",
        )
    );
}

#[test]
fn compiles_translate_axes_and_transitions() {
    let source = r#"
        import { alien, css, tokens } from "alien";
        import { percent, px } from "alien/values";

        const styles = alien.create({
            button: {
                translate: css.translate.none,
                transition: {
                    property: css.transitionProperty.translate,
                    duration: tokens.duration.fast,
                    ease: tokens.ease.inOut,
                },
                on: { hover: { translate: { y: px(-2) } } },
            },
            horizontal: { translate: { x: percent(-50) } },
            depth: { translate: { z: px(8), x: percent(50) } },
        });
    "#;
    let result = compile_typescript(source, Path::new("translate.ts")).unwrap();

    assert_eq!(
        result.css,
        concat!(
            ".a0{translate:none;transition-property:translate;transition-duration:120ms;",
            "transition-timing-function:cubic-bezier(0.42,0,0.58,1)}",
            ".a0:hover{translate:0 -2px}",
            ".a1{translate:-50%}",
            ".a2{translate:50% 0 8px}",
        )
    );
}

#[test]
fn rejects_invalid_translate_axes() {
    let template = r#"
        import { alien } from "alien";
        import { percent, px } from "alien/values";
        const styles = alien.create({ button: { translate: AXES } });
    "#;
    for (axes, diagnostic) in [
        ("{}", "translate requires at least one axis"),
        ("{ y: px(1), y: px(2) }", "duplicate translate.y"),
        ("{ z: percent(20) }", "expected a length helper"),
        ("{ w: px(1) }", "expected a translate axis"),
    ] {
        let error = compile_typescript(&template.replace("AXES", axes), Path::new("translate.ts"))
            .unwrap_err();
        assert!(error.to_string().contains(diagnostic), "{error}");
    }
}

#[test]
fn orders_simple_pseudo_classes_and_combines_nested_conditions() {
    let source = r#"
        import { alien, css, tokens } from "alien";
        import { px } from "alien/values";

        const styles = alien.create({
            button: {
                on: {
                    active: {
                        translate: { y: px(1) },
                        on: { hover: { translate: { y: px(2) } } },
                    },
                    disabled: { cursor: css.cursor.notAllowed },
                    focusVisible: { outlineColor: tokens.color.blue[600] },
                    hover: { translate: { y: px(-2) } },
                    breakpoint: {
                        md: {
                            on: { checked: { backgroundColor: tokens.color.black } },
                        },
                    },
                },
            },
        });
    "#;
    let result = compile_typescript(source, Path::new("button.ts")).unwrap();

    assert_eq!(
        result.css,
        concat!(
            ".a0:hover{translate:0 -2px}",
            ".a0:focus-visible{outline-color:oklch(0.546 0.245 262.881)}",
            ".a0:active{translate:0 1px}",
            ".a0:disabled{cursor:not-allowed}",
            "@media(min-width:48rem){.a0:checked{background-color:oklch(0 0 0)}}",
            ".a0:hover:active{translate:0 2px}",
        )
    );
}

#[test]
fn rejects_repeated_pseudo_classes_and_equivalent_conditions() {
    let template = r#"
        import { alien, tokens } from "alien";
        const styles = alien.create({ button: { on: CONDITIONS } });
    "#;
    for (conditions, diagnostic) in [
        (
            "{ active: { on: { active: {} } } }",
            "cannot repeat `on.active` inside a condition",
        ),
        (
            "{ active: { on: { hover: { color: tokens.color.black } } },
               hover: { on: { active: { color: tokens.color.white } } } }",
            "duplicate color declaration",
        ),
        ("{ focussed: {} }", "unsupported condition `on.focussed`"),
    ] {
        let error = compile_typescript(
            &template.replace("CONDITIONS", conditions),
            Path::new("button.ts"),
        )
        .unwrap_err();
        assert!(error.to_string().contains(diagnostic), "{error}");
    }
}

#[test]
fn applies_alpha_to_palette_and_plain_color_tokens() {
    let source = r#"
        import { alien, tokens as t } from "alien";
        import { alpha as opacity, percent as pct } from "alien/values";

        const styles = alien.create({
            button: {
                backgroundColor: opacity(t.color.black, pct(25)),
                on: {
                    focusVisible: {
                        outlineColor: opacity(t.color.neutral[200], pct(50)),
                        border: { color: opacity(opacity(t.color.green[600], pct(25)), pct(40)) },
                    },
                },
            },
        });
    "#;
    let result = compile_typescript(source, Path::new("button.ts")).unwrap();

    assert_eq!(
        result.css,
        concat!(
            ".a0{background-color:oklch(0 0 0/25%)}",
            ".a0:focus-visible{outline-color:oklch(0.922 0 none/50%);",
            "border-color:oklch(0.627 0.194 149.214/40%)}",
        ),
    );
}

#[test]
fn rejects_invalid_alpha_percentages() {
    let template = r#"
        import { alien, tokens } from "alien";
        import { alpha, percent } from "alien/values";

        const styles = alien.create({
            button: { outlineColor: alpha(tokens.color.neutral[200], OPACITY) },
        });
    "#;
    for opacity in ["percent(-1)", "percent(101)", "50"] {
        let source = template.replace("OPACITY", opacity);
        let error = compile_typescript(&source, Path::new("button.ts")).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("alpha() expects percent() between 0 and 100")
        );
    }
}

#[test]
fn compiles_explicit_transition_property_lists() {
    let source = r#"
        import { alien, css, tokens } from "alien";
        const styles = alien.create({
            button: {
                transition: {
                    property: [css.transitionProperty.backgroundColor, css.transitionProperty.translate],
                    duration: tokens.duration.fast,
                },
            },
        });
    "#;
    let result = compile_typescript(source, Path::new("button.ts")).unwrap();

    assert_eq!(
        result.css,
        ".a0{transition-property:background-color,translate;transition-duration:120ms}"
    );
}

#[test]
fn rejects_empty_or_unscoped_transition_property_lists() {
    let template = r#"
        import { alien, css } from "alien";
        const styles = alien.create({ button: { transition: { property: TARGETS } } });
    "#;
    for (targets, diagnostic) in [
        ("[]", "expected a non-empty array"),
        (
            "[css.transitionProperty.backgroundColor, css.transitionProperty.all]",
            "all and none must be used alone",
        ),
        (
            "[css.transitionProperty.none]",
            "all and none must be used alone",
        ),
    ] {
        let source = template.replace("TARGETS", targets);
        let error = compile_typescript(&source, Path::new("button.ts")).unwrap_err();
        assert!(error.to_string().contains(diagnostic), "{error}");
    }
}

#[test]
fn compiles_custom_hex_and_rejects_invalid_values() {
    let template = r#"
        import { alien, custom } from "alien";
        import { hex } from "alien/values";
        const styles = alien.create({ badge: { backgroundColor: custom(hex("COLOR")) } });
    "#;
    let compiled =
        compile_typescript(&template.replace("COLOR", "#ff00aa"), Path::new("badge.ts")).unwrap();
    assert_eq!(compiled.css, ".a0{background-color:#ff00aa}");
    let error =
        compile_typescript(&template.replace("COLOR", "#xyz"), Path::new("badge.ts")).unwrap_err();
    assert!(error.to_string().contains("valid CSS hexadecimal color"));
}

#[test]
fn shares_adjacent_identical_styles_without_reordering_the_cascade() {
    let source = r#"
        import { alien } from "alien";
        const styles = alien.create({
            first: { display: "flex" },
            second: { display: "flex" },
            third: { display: "grid" },
            fourth: { display: "flex" },
        });
    "#;
    let output = compile_typescript(source, Path::new("test.ts")).unwrap();
    assert_eq!(output.classes["first"], output.classes["second"]);
    assert_ne!(output.classes["first"], output.classes["fourth"]);
    assert_eq!(
        output.css,
        ".a0{display:flex}.a2{display:grid}.a3{display:flex}"
    );
}

#[test]
fn shares_profitable_atomic_declarations_and_keeps_different_values_grouped() {
    let entries = (1..=20)
        .map(|index| format!("s{index}:{{display:'flex',width:px({index})}}"))
        .collect::<Vec<_>>()
        .join(",");
    let source = format!(
        "import {{alien}} from 'alien'; import {{px}} from 'alien/values';const styles=alien.create({{{entries}}});"
    );
    let output = compile_typescript(&source, Path::new("styles.ts")).unwrap();
    assert_eq!(output.css.matches("display:flex").count(), 1);
    assert!(output.classes.values().all(|class| class.contains("au0_")));
    assert!(
        output
            .classes
            .values()
            .all(|class| class.split_whitespace().count() == 2)
    );
    assert!(output.css.contains("width:1px"));
    assert!(output.css.contains("width:20px"));
}

#[test]
fn keeps_conflicting_conditional_declarations_in_original_order() {
    let entries = (0..20)
        .map(|index| format!("s{index}:{{display:'flex',on:{{hover:{{display:'grid'}}}}}}"))
        .collect::<Vec<_>>()
        .join(",");
    let source = format!("import {{alien}} from 'alien';const styles=alien.create({{{entries}}});");
    let output = compile_typescript(&source, Path::new("styles.ts")).unwrap();
    assert!(!output.classes.values().any(|class| class.contains("au0_")));
    assert_eq!(output.css.matches("display:grid").count(), 1);
}

#[test]
fn preserves_shorthand_conditional_and_important_conflicts_while_sharing_safe_rules() {
    for conflict in [
        "on:{breakpoint:{md:{paddingLeft:'8px'}}}",
        "on:{hover:{paddingInline:'8px'}}",
        "on:{breakpoint:{md:{padding:important('4px')}}}",
    ] {
        let entries = (0..30)
            .map(|i| format!("s{i}:{{display:'flex',padding:'4px',width:px({i}),{conflict}}}"))
            .collect::<Vec<_>>()
            .join(",");
        let source = format!(
            "import {{alien}} from 'alien';import {{px,important}} from 'alien/values';const s=alien.create({{{entries}}});"
        );
        let output = compile_typescript(&source, Path::new("conflicts.ts")).unwrap();
        assert_eq!(output.css.matches("display:flex").count(), 1);
        // All original padding declarations survive, including their priority.
        assert_eq!(
            output.css.matches("padding:4px}").count() + output.css.matches("padding:4px;").count(),
            30
        );
        assert_eq!(
            output.css.matches("padding:4px!important").count(),
            if conflict.contains("important") {
                30
            } else {
                0
            }
        );
    }
}

#[test]
fn transformation_reports_readable_source_locations_and_limits_nested_lengths() {
    let expression = (0..66).fold("px(1)".to_owned(), |inner, _| {
        format!("calc.sub({inner},px(1))")
    });
    let source = format!(
        "import {{alien}} from 'alien';\nimport {{calc,px}} from 'alien/values';\nconst s=alien.create({{a:{{width:{expression}}}}});"
    );
    let error = transform_typescript(&source, Path::new("nested.ts"), "nested.ts")
        .unwrap_err()
        .to_string();
    assert!(error.contains("length expressions cannot exceed 64 levels"));
    assert!(error.contains("nested.ts"));
    assert!(error.contains("line 3, column"));
}

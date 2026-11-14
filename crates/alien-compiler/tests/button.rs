use alien_compiler::{CompileError, compile_styles};
use alien_ir::{
    ColorToken, ColorValue, Condition, Declaration, Property, PseudoClass, Rule, Style, Value,
};

fn background(condition: Condition, step: u16) -> Rule {
    Rule {
        condition,
        declaration: Declaration {
            property: Property::BackgroundColor,
            important: false,
            value: Value::Color(ColorValue::Token(ColorToken::Shade {
                palette: "green".into(),
                step,
            })),
        },
    }
}

#[test]
fn compiles_a_button_with_a_hover_color() {
    let mut hover = Condition::default();
    hover.pseudo_classes.insert(PseudoClass::Hover);
    let button = Style {
        name: "button".into(),
        rules: vec![
            background(hover, 700),
            background(Condition::default(), 600),
        ],
    };

    let output = compile_styles(&[button]).unwrap();

    assert_eq!(output.classes["button"], "a0");
    assert_eq!(
        output.css,
        concat!(
            ".a0{background-color:oklch(0.627 0.194 149.214)}",
            ".a0:hover{background-color:oklch(0.527 0.154 150.069)}",
        )
    );
}

#[test]
fn reports_an_unknown_shade() {
    let button = Style {
        name: "button".into(),
        rules: vec![background(Condition::default(), 601)],
    };

    assert_eq!(
        compile_styles(&[button]).unwrap_err(),
        CompileError::UnknownColorShade {
            palette: "green".into(),
            step: 601,
        }
    );
}

#[test]
fn rejects_duplicate_style_names() {
    let styles = std::array::from_fn::<_, 2, _>(|_| Style {
        name: "button".into(),
        rules: vec![background(Condition::default(), 600)],
    });

    assert_eq!(
        compile_styles(&styles).unwrap_err(),
        CompileError::DuplicateStyle("button".into())
    );
}

#[test]
fn repeated_ir_declarations_remain_valid_without_affecting_optimizer_costs() {
    let styles: Vec<_> = (0..20)
        .map(|index| Style {
            name: format!("s{index}"),
            rules: vec![
                background(Condition::default(), 600),
                background(Condition::default(), 600),
            ],
        })
        .collect();
    let output = compile_styles(&styles).unwrap();
    assert_eq!(output.css.matches("background-color:").count(), 2);
    assert!(output.classes.values().all(|class| class == "a0"));
}

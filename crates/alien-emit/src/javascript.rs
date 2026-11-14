use std::collections::BTreeMap;

/// Appends the JavaScript object that replaces an `alien.create()` call.
pub fn write_class_map(output: &mut String, classes: &BTreeMap<String, String>) {
    output.push_str("({");
    for (index, (name, class)) in classes.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        // A computed key preserves "__proto__" as an ordinary property.
        let computed = name == "__proto__";
        if computed {
            output.push('[');
        }
        output.push_str(&quote(name));
        if computed {
            output.push(']');
        }
        output.push(':');
        output.push_str(&quote(class));
    }
    output.push_str("})");
}

fn quote(value: &str) -> String {
    serde_json::to_string(value).expect("serializing a string to JSON cannot fail")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::write_class_map;

    #[test]
    fn ordinary_keys_do_not_need_computed_property_overhead() {
        let classes = BTreeMap::from([
            ("button".to_owned(), "a0".to_owned()),
            ("with space".to_owned(), "a1".to_owned()),
        ]);
        let mut output = String::new();
        write_class_map(&mut output, &classes);
        assert_eq!(output, r#"({"button":"a0","with space":"a1"})"#);
    }

    #[test]
    fn writes_proto_as_a_computed_key_and_escapes_strings() {
        let classes = BTreeMap::from([("__proto__".to_owned(), "a\"quoted".to_owned())]);
        let mut output = String::new();

        write_class_map(&mut output, &classes);

        assert_eq!(output, r#"({["__proto__"]:"a\"quoted"})"#);
    }
}

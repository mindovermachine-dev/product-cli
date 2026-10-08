//! The scalars the second reading refuses, against the ones it leaves as text.

use super::faults;

fn statement(scalar: &str) -> String {
    format!("format: 1\nid: cs:01A\nversions:\n  - decision: dec:ns/01C\n    statement: {scalar}\n    set: s\n")
}

#[test]
fn a_plain_float_in_any_spelling_is_refused() {
    for s in ["1.5", "1.50", "+1.5", "1e3", "-2.5E-3", ".inf", "-.inf", "+.inf", ".Inf", ".INF", ".nan", ".NaN", ".5", "1."] {
        let f = faults("f.yml", &statement(s));
        assert_eq!(f.len(), 1, "{s}: {f:?}");
        assert!(f[0].message.contains("versions[0].statement"), "{s}: {f:?}");
    }
}

#[test]
fn quoted_floats_integers_and_words_are_text() {
    for s in ["'1.5'", "\"1e3\"", "42", "-7", "Money is decimal.", "inf", "nan", "1.5.2", "true"] {
        assert!(faults("f.yml", &statement(s)).is_empty(), "{s}");
    }
}

#[test]
fn an_explicit_null_in_a_required_string_field_is_refused() {
    for s in ["null", "~", "Null", "NULL"] {
        let f = faults("f.yml", &statement(s));
        assert_eq!(f.len(), 1, "{s}: {f:?}");
        assert!(f[0].message.contains("explicit null"), "{s}: {f:?}");
    }
    assert!(faults("f.yml", &statement("'null'")).is_empty(), "a quoted null is the text null");
}

#[test]
fn a_null_in_an_optional_field_is_absent_and_fine() {
    let text = statement("Money is decimal.").replace("    set: s\n", "    set: s\n    exposure: null\n");
    assert!(faults("f.yml", &text).is_empty());
}

#[test]
fn floats_are_found_in_nested_lists_and_outside_versions() {
    let text = "format: 7\nid: cs:01A\nrevocations:\n  - id: rev:01B\n    reason: 2.0\npolicies:\n  - id: pol:01C\n    schemes: [1.5]\n";
    let f = faults("f.yml", text);
    assert_eq!(f.len(), 2, "{f:?}");
}

#[test]
fn the_header_and_decision_identity_objects_are_not_hashed() {
    let text = "format: 1\nid: cs:01A\nnote: 1.5\ndecisions:\n  - id: dec:ns/01C\n    created_by: 2.5\n";
    assert!(faults("f.yml", text).is_empty());
}

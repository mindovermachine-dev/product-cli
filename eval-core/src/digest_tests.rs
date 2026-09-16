use super::*;

fn shown(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs.iter().map(|(k, v)| ((*k).to_string(), (*v).to_string())).collect()
}

#[test]
fn the_same_context_pins_the_same_digest() {
    assert_eq!(pin(&shown(&[("a", "1")])), pin(&shown(&[("a", "1")])));
}

#[test]
fn a_different_context_pins_a_different_digest() {
    assert_ne!(pin(&shown(&[("a", "1")])), pin(&shown(&[("a", "2")])));
}

/// Order is formatting, not meaning: the map is sorted before it is hashed.
#[test]
fn key_order_is_not_part_of_the_context() {
    let one = shown(&[("a", "1"), ("b", "2")]);
    let other = shown(&[("b", "2"), ("a", "1")]);
    assert_eq!(pin(&one), pin(&other));
}

/// Absent and present-but-empty must not pin differently.
#[test]
fn an_empty_value_is_omitted_rather_than_hashed() {
    assert_eq!(pin(&shown(&[("a", "1"), ("b", "   ")])), pin(&shown(&[("a", "1")])));
}

#[test]
fn line_endings_are_folded_before_hashing() {
    assert_eq!(pin(&shown(&[("a", "one\r\ntwo")])), pin(&shown(&[("a", "one\ntwo")])));
}

#[test]
fn a_digest_is_domain_separated_and_hex() {
    let digest = pin(&shown(&[("a", "1")]));
    assert!(digest.starts_with("sha256:"), "{digest}");
    assert_eq!(digest.len(), "sha256:".len() + 64);
}

#[test]
fn a_context_holds_its_own_digest() {
    let context = shown(&[("a", "1")]);
    let digest = pin(&context);
    assert!(holds(&context, &digest));
    assert!(!holds(&shown(&[("a", "2")]), &digest));
}

/// The value the .NET half is held to. Changing it is a format change.
#[test]
fn the_pinned_form_is_fixed() {
    assert_eq!(
        pin(&shown(&[("act_ref", "act/settle-a-basket"), ("proposed", "det/a,det/b")])),
        "sha256:78886a7fb45787b072bdcb934b6ce3d67bf20009a57d674cef59c7e6795948e2"
    );
}

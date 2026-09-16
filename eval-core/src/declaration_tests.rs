use super::*;

#[test]
fn a_full_declaration_is_missing_nothing() {
    let declared = Declaration::new("settle the basket", vec!["act/settles".into()])
        .bounded_by("one penny", "high");
    assert!(declared.missing().is_empty());
}

#[test]
fn a_declaration_without_ground_is_incomplete() {
    let declared = Declaration::new("settle the basket", vec![]).bounded_by("τ", "α");
    assert_eq!(declared.missing(), ["ground"]);
}

/// The arrangement's fields, absent because nobody supplied them.
#[test]
fn a_worker_only_declaration_is_missing_the_arrangements_half() {
    let declared = Declaration::new("settle it", vec!["g".into()]);
    assert_eq!(declared.missing(), ["tolerance", "assurance"]);
}

#[test]
fn whitespace_is_not_a_declaration() {
    let declared = Declaration::new("   ", vec!["g".into()]).bounded_by("  ", "α");
    assert!(declared.missing().contains(&"decision"));
    assert!(declared.missing().contains(&"tolerance"));
}

#[test]
fn an_attribution_names_its_ground_or_says_it_has_none() {
    assert_eq!(Attribution::to("c", "g").ground.as_deref(), Some("g"));
    assert_eq!(Attribution::unattributed("c").ground, None);
}

//! The digests both runtimes are held to.
//!
//! `docs/eval-format-v1.md` §6 states the canonical form normatively, and this
//! file is what stops the statement drifting from either implementation. The
//! .NET half asserts against the same JSON, so a change made on one side and
//! not the other fails on both.
//!
//! Regenerate with `UPDATE_FIXTURES=1 cargo test -p eval-core --test fixture`,
//! and read a changed digest as a format change rather than a test to update.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_json::Value;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/context-digest.json")
}

fn load() -> Value {
    let text = std::fs::read_to_string(fixture_path()).expect("the fixture is readable");
    serde_json::from_str(&text).expect("the fixture is json")
}

fn shown_of(case: &Value) -> BTreeMap<String, String> {
    case["shown"]
        .as_object()
        .expect("a case shows an object")
        .iter()
        .map(|(k, v)| (k.clone(), v.as_str().unwrap_or_default().to_string()))
        .collect()
}

#[test]
fn every_case_pins_the_digest_it_claims() {
    let mut fixture = load();
    let updating = std::env::var_os("UPDATE_FIXTURES").is_some();
    let mut mismatched = Vec::new();

    let cases = fixture["cases"].as_array_mut().expect("cases is an array");
    for case in cases.iter_mut() {
        let name = case["name"].as_str().unwrap_or("(unnamed)").to_string();
        let computed = eval_core::digest::pin(&shown_of(case));
        let claimed = case["digest"].as_str().unwrap_or_default().to_string();

        if computed == claimed {
            continue;
        }
        if updating {
            case["digest"] = Value::String(computed);
        } else {
            mismatched.push(format!("  {name}\n    claimed  {claimed}\n    computed {computed}"));
        }
    }

    if updating {
        let text = serde_json::to_string_pretty(&fixture).expect("encodes");
        std::fs::write(fixture_path(), format!("{text}\n")).expect("the fixture is writable");
        return;
    }

    assert!(
        mismatched.is_empty(),
        "the pinned form changed, which is a format change and not a test to update:\n{}",
        mismatched.join("\n")
    );
}

/// The fixture is only worth having if it covers what the doc promises.
#[test]
fn the_fixture_covers_each_rule_the_format_states() {
    let fixture = load();
    let names: Vec<&str> =
        fixture["cases"].as_array().expect("cases").iter().filter_map(|c| c["name"].as_str()).collect();

    for rule in ["omitted", "fold", "NFC", "trimmed", "nothing shown"] {
        assert!(
            names.iter().any(|n| n.contains(rule)),
            "no case covers `{rule}`; §6 states it, so something must hold it"
        );
    }
}

/// A case that normalises to itself proves nothing about normalisation.
///
/// Regenerating the fixture rewrites its JSON, and a writer that composed the
/// decomposed input would leave the NFC case passing vacuously. This fails
/// instead, and names why.
#[test]
fn the_cases_that_test_normalisation_still_need_it() {
    let fixture = load();
    let cases = fixture["cases"].as_array().expect("cases");

    for rule in ["NFC", "fold", "trimmed"] {
        let case = cases
            .iter()
            .find(|c| c["name"].as_str().is_some_and(|n| n.contains(rule)))
            .unwrap_or_else(|| panic!("a case covering `{rule}`"));

        let raw: Vec<String> = shown_of(case).into_values().collect();
        assert!(
            raw.iter().any(|v| ledger_core::canon::norm(v).as_deref() != Some(v.as_str())),
            "the `{rule}` case no longer holds input that normalisation changes"
        );
    }
}

use super::*;

#[test]
fn a_host_is_kept_and_the_rest_of_the_url_is_not() {
    assert_eq!(RunRecord::host_of("https://api.scaleway.ai/v1").as_deref(), Some("api.scaleway.ai"));
    assert_eq!(RunRecord::host_of("http://192.168.88.63:8000").as_deref(), Some("192.168.88.63"));
}

/// A key pasted into a URL must not reach the store through it.
#[test]
fn credentials_in_a_url_are_not_kept() {
    let host = RunRecord::host_of("https://user:secret@api.example.com/v1?key=abc123");
    assert_eq!(host.as_deref(), Some("api.example.com"));
}

#[test]
fn something_that_is_not_a_url_yields_no_host() {
    assert_eq!(RunRecord::host_of("not a url"), None);
}

#[test]
fn a_draft_nobody_touched_was_not_amended() {
    let mut run = RunRecord::new("01ABC", "spec-flow", "act/a", "slice");
    run.proposed = vec!["det/a".into()];
    run.kept.clone_from(&run.proposed);
    assert!(!run.was_amended());
}

#[test]
fn a_draft_the_reviewer_cut_was_amended() {
    let mut run = RunRecord::new("01ABC", "spec-flow", "act/a", "slice");
    run.proposed = vec!["det/a".into()];
    assert!(run.was_amended());
}

/// The format doc's shape, held to by `deny_unknown_fields` in both directions.
#[test]
fn a_run_record_round_trips() {
    let mut run = RunRecord::new("01ABC", "spec-flow", "act/a", "slice");
    run.model = Some("qwen3.6-35b-a3b".into());
    run.endpoint_host = RunRecord::host_of("https://api.scaleway.ai/v1");
    run.proposed = vec!["det/a".into()];
    run.reply = Some("built it".into());
    run.metrics = vec![Metric::new("Reviewer amendment", Some("0".into()), "kept").rated("Good")];

    let text = serde_json::to_string(&run).expect("encodes");
    assert_eq!(serde_json::from_str::<RunRecord>(&text).expect("decodes"), run);
    assert!(!text.contains("secret"), "no credential field exists to leak");
}

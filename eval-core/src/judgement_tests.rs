use super::*;

fn shown() -> BTreeMap<String, String> {
    [("task".to_string(), "act/settle-a-basket".to_string())].into_iter().collect()
}

#[test]
fn a_judge_is_named_as_a_machine() {
    let judge = Judge::new("glm-5.2", Some("api.scaleway.ai".into()));
    assert_eq!(judge.identity, "model:glm-5.2");
}

#[test]
fn a_judgement_ratifies_nothing() {
    let judgement = Judgement::new("01ABC", Judge::new("glm-5.2", None), shown(), vec![]);
    assert!(judgement.ratifies_nothing);
}

/// No principal field exists to be filled, in the type or on the wire.
#[test]
fn nothing_on_a_judgement_names_a_principal() {
    let judgement = Judgement::new("01ABC", Judge::new("glm-5.2", None), shown(), vec![]);
    let text = serde_json::to_string(&judgement).expect("encodes");
    assert!(!text.contains("principal"), "{text}");
    assert!(text.contains("\"ratifies_nothing\":true"), "{text}");
}

#[test]
fn a_judgement_pins_its_context_as_it_is_built() {
    let judgement = Judgement::new("01ABC", Judge::new("glm-5.2", None), shown(), vec![]);
    assert!(judgement.context_holds());
}

/// A context edited after the fact no longer holds its digest.
#[test]
fn an_altered_context_does_not_hold() {
    let mut judgement = Judgement::new("01ABC", Judge::new("glm-5.2", None), shown(), vec![]);
    judgement.context.shown.insert("task".into(), "act/something-else".into());
    assert!(!judgement.context_holds());
}

#[test]
fn a_judgement_round_trips() {
    let judgement = Judgement::new(
        "01ABC",
        Judge::new("glm-5.2", Some("api.scaleway.ai".into())),
        shown(),
        vec![Metric::new("Determination warrant", Some("4/5".into()), "one describes work")
            .rated("Average")],
    );
    let text = serde_json::to_string(&judgement).expect("encodes");
    assert_eq!(serde_json::from_str::<Judgement>(&text).expect("decodes"), judgement);
}

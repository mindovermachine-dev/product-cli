use super::*;

#[test]
fn a_path_is_a_disk_store() {
    assert_eq!(Backend::parse(".spec").expect("parses"), Backend::disk(".spec"));
}

#[test]
fn an_azure_spelling_names_an_account_and_a_container() {
    assert_eq!(
        Backend::parse("azure:evalstore/runs").expect("parses"),
        Backend::Azure {
            account: "evalstore".into(),
            container: "runs".into(),
            prefix: String::new()
        }
    );
}

#[test]
fn a_prefix_lets_one_container_hold_several_tools() {
    let parsed = Backend::parse("azure:evalstore/runs/spec-flow").expect("parses");
    assert_eq!(
        parsed,
        Backend::Azure {
            account: "evalstore".into(),
            container: "runs".into(),
            prefix: "spec-flow".into()
        }
    );
}

#[test]
fn a_half_written_azure_spelling_is_refused_rather_than_guessed_at() {
    assert!(Backend::parse("azure:evalstore").is_err());
    assert!(Backend::parse("azure:").is_err());
    assert!(Backend::parse("azure:/runs").is_err());
}

#[test]
fn disk_opens() {
    let dir = tempfile::tempdir().expect("tempdir");
    assert!(Backend::disk(dir.path()).open().is_ok());
}

/// A tool told to write to Azure must not quietly write to disk instead.
#[test]
fn azure_says_it_is_not_built_rather_than_falling_back() {
    let declared = Backend::parse("azure:evalstore/runs").expect("parses");
    let Err(refusal) = declared.open() else {
        panic!("azure is not built; opening it must not succeed");
    };
    assert!(format!("{refusal}").contains("not built"), "{refusal}");
}

/// The config round-trips, so a file can name a backend a tool will accept.
#[test]
fn a_backend_round_trips_through_config() {
    for backend in [Backend::disk(".spec"), Backend::parse("azure:a/c/p").expect("parses")] {
        let text = serde_json::to_string(&backend).expect("encodes");
        assert_eq!(serde_json::from_str::<Backend>(&text).expect("decodes"), backend);
    }
}

#[test]
fn a_backend_describes_itself_for_a_log_line() {
    assert!(Backend::disk(".spec").describe().starts_with("disk at"));
    assert_eq!(
        Backend::parse("azure:a/c").expect("parses").describe(),
        "azure a/c"
    );
}

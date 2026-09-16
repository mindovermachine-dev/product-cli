use super::{build_forwarded, executable_name, guard, import_forwarded, BuildArgs, ImportArgs};
use std::path::PathBuf;

fn build_args() -> BuildArgs {
    BuildArgs {
        slice: "checkout-totals".into(),
        act_ref: "act/settle-basket".into(),
        by: None,
        instructions: None,
    }
}

fn flattened(args: &[std::ffi::OsString]) -> Vec<String> {
    args.iter().map(|a| a.to_string_lossy().into_owned()).collect()
}

#[test]
fn a_verb_naming_a_principal_is_never_delegated() {
    for verb in ["accept", "reject", "close", "policy"] {
        assert!(guard(verb).is_err(), "`{verb}` names a principal and must not reach the host");
    }
}

#[test]
fn the_delegable_verbs_pass_the_guard() {
    assert!(guard("import").is_ok());
    assert!(guard("implement").is_ok());
}

#[test]
fn build_forwards_the_slice_it_was_given() {
    assert_eq!(
        flattened(&build_forwarded(&build_args())),
        ["--slice", "checkout-totals", "--act", "act/settle-basket"]
    );
}

#[test]
fn an_absent_option_forwards_no_flag() {
    let forwarded = flattened(&build_forwarded(&build_args()));
    assert!(!forwarded.contains(&"--by".to_string()));
    assert!(!forwarded.contains(&"--instructions".to_string()));
}

#[test]
fn a_supplied_option_forwards_as_a_pair() {
    let args = BuildArgs { by: Some("agent@example.invalid".into()), ..build_args() };
    let forwarded = flattened(&build_forwarded(&args));
    let at = forwarded.iter().position(|a| a == "--by").expect("the flag is forwarded");
    assert_eq!(forwarded.get(at + 1).map(String::as_str), Some("agent@example.invalid"));
}

#[test]
fn import_forwards_nothing_when_the_source_is_the_root() {
    assert!(import_forwarded(&ImportArgs { source: None }).is_empty());
}

#[test]
fn import_forwards_a_source_when_one_is_given() {
    let args = ImportArgs { source: Some(PathBuf::from("src/Api")) };
    assert_eq!(flattened(&import_forwarded(&args)), ["--source", "src/Api"]);
}

#[test]
fn the_host_carries_this_platforms_suffix() {
    assert_eq!(executable_name(), format!("specflow{}", std::env::consts::EXE_SUFFIX));
}

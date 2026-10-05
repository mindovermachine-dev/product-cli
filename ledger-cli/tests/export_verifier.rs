//! The export-only verifier (#70): input fixed to the export file and the
//! sidecar files — no `.decisions/` entity file, no git — and the property
//! that bytes rebuilt from the export equal the bytes that were signed.

mod common;

use common::export_only::{allowed_signers, parse, rebuild, verify_all};
use common::{hand, Repo};

const OWNER: &str = "owner@customer.example";
const ARCHITECT: &str = "architect@customer.example";
const NS: &str = "fixture.ledger";

/// A store with every signable kind: self-bound and vouched bindings, a
/// rotation, an acceptance, a revocation of an acceptance, a policy change.
fn signed_store() -> Repo {
    let repo = Repo::with_identity(OWNER);
    repo.declare();
    repo.ok(&["init", "--namespace", NS, "--external-ref", "contract 2026/117", "--without-key"]);
    repo.bind_own_key(NS, "owner");
    let grant = hand::word(&repo.ok(&["grant", "new", "acceptor", "--to", OWNER, "--scope", &format!("ns:{NS}")]), "grant:");
    repo.ok(&["grant", "accept", &grant]);
    repo.vouch_for(NS, ARCHITECT, "architect");
    let first = repo.add("Money is decimal.", &[]);
    let second = repo.add("Time is UTC.", &["--key", "TimeIsUtc"]);
    repo.ok_tty(&["accept", &first, "--expires", "2099-01-01"]);
    repo.ok_tty(&["accept", &second]);
    let store = ledger_core::store::load(repo.path());
    let acc = store.log.iter().flat_map(|l| l.file.acceptances.iter()).map(|a| a.id.to_string()).next().expect("acc");
    repo.ok_tty(&["revoke", &acc, "--reason", "filed against \"the\" wrong version"]);
    repo.ok(&["policy", "set", "--namespace", NS, "--reaccept-within-days", "14"]);
    let owner_binding = store
        .log
        .iter()
        .flat_map(|l| l.file.key_bindings.iter())
        .find(|b| b.principal.as_str() == OWNER)
        .map(|b| b.id.to_string())
        .expect("owner binding");
    let next = repo.keygen("owner-next");
    repo.ok(&["identity", "rotate", &owner_binding, "--key-file", &format!("{next}.pub")]);
    repo.ok(&["verify", "--no-blame"]);
    repo
}

#[test]
fn the_export_and_the_sidecars_alone_verify_and_rebuild_allowed_signers() {
    let repo = signed_store();
    let input = tempfile::tempdir().expect("tempdir");
    let export = input.path().join(format!("{NS}.nt"));
    repo.ok(&["export", "--format", "ntriples", "--namespace", NS, "--out", &export.display().to_string()]);
    std::fs::create_dir_all(input.path().join("sig")).expect("sig dir");
    let mut sidecars = 0;
    for entry in std::fs::read_dir(repo.path().join(".decisions/sig")).expect("sig").flatten() {
        std::fs::copy(entry.path(), input.path().join("sig").join(entry.file_name())).expect("copy");
        sidecars += 1;
    }
    let committed = std::fs::read_to_string(repo.path().join(".decisions/allowed_signers")).expect("derived");
    // The verifier's input is fixed: nothing under `.decisions/`, no git.
    std::fs::remove_dir_all(repo.path().join(".decisions")).expect("remove store");
    std::fs::remove_dir_all(repo.path().join(".git")).expect("remove git");

    let graph = parse(&std::fs::read_to_string(&export).expect("export"));
    assert_eq!(allowed_signers(&graph), committed, "rebuilt byte for byte from the KeyBinding nodes");
    match verify_all(&graph, input.path()) {
        Ok(n) => assert_eq!(n, sidecars, "every sidecar is referenced and verified"),
        Err(failures) => panic!("export-only verification failed: {failures:#?}"),
    }
}

#[test]
fn bytes_rebuilt_from_the_export_equal_the_signed_bytes_over_every_fixture() {
    let signed = signed_store();
    let fixtures = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut roots: Vec<std::path::PathBuf> = std::fs::read_dir(&fixtures).expect("fixtures").flatten().map(|e| e.path()).collect();
    roots.sort();
    roots.push(signed.path().to_path_buf());
    // And this repository's own store: 91 acceptances signed by nobody yet,
    // whose bytes the export must still rebuild exactly.
    roots.push(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".."));
    let mut checked = 0;
    for root in roots {
        let store = ledger_core::store::load(&root);
        let subjects = ledger_core::signing::subject::subjects(&store, &ledger_core::landing::Landing::unknown());
        for ns in ledger_core::graph::export::namespaces(&store) {
            let Some(text) = ledger_core::graph::export::export(&store, &ns) else { continue };
            let graph = parse(&text);
            for s in subjects.iter().filter(|s| s.namespace == ns) {
                let rebuilt = rebuild(&graph, &format!("urn:{}", s.id));
                assert_eq!(rebuilt.as_deref(), Some(s.bytes.as_slice()), "{}: {} in `{ns}`", root.display(), s.id);
                checked += 1;
            }
        }
    }
    assert!(checked > 100, "the property must cover the fixtures and the real store: {checked}");
}

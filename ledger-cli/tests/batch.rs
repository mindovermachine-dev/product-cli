//! `ledger accept --batch <file>` (#86): a selection file signed row by row,
//! pinned by its manifest, one signature per acceptance.

mod common;

use common::hand;
use common::Repo;
use ledger_core::batch_file::{BatchFile, Row, BATCH_FORM};

const OWNER: &str = "owner@customer.example";
const NS: &str = "fixture.ledger";

/// A governed namespace, the owner's key bound, the owner granted the
/// accept role. Committed.
fn governed() -> Repo {
    let repo = Repo::with_identity(OWNER);
    repo.declare();
    repo.ok(&["init", "--namespace", NS, "--external-ref", "contract 2026/117"]);
    repo.bind_own_key(NS, "owner");
    let grant = hand::word(&repo.ok(&["grant", "new", "acceptor", "--to", OWNER, "--scope", &format!("ns:{NS}")]), "grant:");
    repo.ok(&["grant", "accept", &grant]);
    hand::commit(&repo, "governed");
    repo
}

/// A batch over `decisions` at their current tips, written to the repo.
fn batch(repo: &Repo, decisions: &[String], as_role: Option<&str>) -> String {
    let file = BatchFile {
        form: BATCH_FORM.to_string(),
        actor: OWNER.to_string(),
        as_role: as_role.map(str::to_string),
        rows: decisions
            .iter()
            .map(|d| Row { repository: "here".into(), branch: None, decision: d.clone(), version: hand::tip(repo, d), grant: None })
            .collect(),
    };
    let path = repo.path().join("batch.yml");
    ledger_core::batch_file::write(&path, &file).expect("write batch");
    path.display().to_string()
}

fn manifest(out: &str) -> String {
    out.lines()
        .find_map(|l| l.strip_prefix("manifest"))
        .map(|m| m.trim().to_string())
        .unwrap_or_else(|| panic!("no manifest in {out}"))
}

fn sidecars(repo: &Repo) -> usize {
    std::fs::read_dir(repo.path().join(".decisions/sig")).map(|d| d.count()).unwrap_or(0)
}

fn three(repo: &Repo) -> Vec<String> {
    ["Money is decimal.", "Time is UTC.", "Ids are ULIDs."].iter().map(|s| repo.add(s, &[])).collect()
}

#[test]
fn a_batch_signs_every_row_one_sidecar_each_under_one_confirmation() {
    let repo = governed();
    let ids = three(&repo);
    let file = batch(&repo, &ids, None);
    let before = sidecars(&repo);
    let dry = repo.ok(&["accept", "--batch", &file]);
    assert!(dry.contains("dry run — nothing written"), "{dry}");
    assert!(dry.matches("under grant:").count() == 3, "the grant per row: {dry}");
    let m = manifest(&dry);
    let out = repo.ok_tty(&["accept", "--batch", &file, "--confirm", &m]);
    assert!(out.contains("signed 3 decision(s)"), "{out}");
    assert_eq!(sidecars(&repo) - before, 3, "one sidecar per acceptance");
    let store = ledger_core::store::load(repo.path());
    assert_eq!(store.log.iter().flat_map(|l| l.file.acceptances.iter()).count(), 3);
    let verify = repo.ledger(&["verify", "--no-blame"]);
    assert_eq!(verify.status.code(), Some(0), "{}", common::both(&verify));
}

#[test]
fn a_moved_hash_refuses() {
    let repo = governed();
    let ids = three(&repo);
    let file = batch(&repo, &ids, None);
    let m = manifest(&repo.ok(&["accept", "--batch", &file]));
    repo.ok(&["revise", &ids[1], "--statement", "Time is UTC, always."]);
    let out = repo.refused_tty(&["accept", "--batch", &file, "--confirm", &m]);
    assert!(out.contains("refused") && out.contains(&ids[1]) && out.contains("moved"), "{out}");
    assert_eq!(sidecars(&repo), 1, "only the owner's key binding is signed: nothing new");
}

#[test]
fn a_missing_row_refuses() {
    let repo = governed();
    let mut ids = three(&repo);
    ids.push(format!("dec:{NS}/01K9ZZZZZZZZZZZZZZZZZZZZZZ"));
    let file = BatchFile {
        form: BATCH_FORM.to_string(),
        actor: OWNER.to_string(),
        as_role: None,
        rows: ids
            .iter()
            .map(|d| Row {
                repository: "here".into(),
                branch: None,
                decision: d.clone(),
                version: if d.ends_with("ZZZZ") { format!("sha256:{}", "0".repeat(64)) } else { hand::tip(&repo, d) },
                grant: None,
            })
            .collect(),
    };
    let path = repo.path().join("batch.yml");
    ledger_core::batch_file::write(&path, &file).expect("write");
    let out = repo.refused(&["accept", "--batch", &path.display().to_string()]);
    assert!(out.contains("missing") && out.contains("01K9ZZZZZZZZZZZZZZZZZZZZZZ"), "{out}");
}

#[test]
fn a_row_with_no_qualifying_grant_under_as_refuses_the_batch() {
    let repo = governed();
    let ids = three(&repo);
    // The owner holds the genesis role, but the namespace accepts under
    // `acceptor`: as `steward` no row has a qualifying grant.
    let file = batch(&repo, &ids, Some("steward"));
    let out = repo.refused(&["accept", "--batch", &file]);
    assert!(out.contains("refused") && out.contains("cannot be accepted"), "{out}");
    assert!(out.contains("steward"), "{out}");
}

#[test]
fn piped_stdin_refuses_and_writes_nothing() {
    let repo = governed();
    let ids = three(&repo);
    let file = batch(&repo, &ids, None);
    let m = manifest(&repo.ok(&["accept", "--batch", &file]));
    let before = (repo.log_files(), sidecars(&repo));
    let out = repo.piped(&["accept", "--batch", &file, "--confirm", &m]);
    assert_ne!(out.status.code(), Some(0), "{}", common::both(&out));
    assert_eq!((repo.log_files(), sidecars(&repo)), before, "nothing written");
}

#[test]
fn another_principals_batch_is_refused() {
    let repo = governed();
    let ids = three(&repo);
    let file = batch(&repo, &ids, None);
    let text = std::fs::read_to_string(&file).expect("read").replace(OWNER, "someone-else@customer.example");
    std::fs::write(&file, text).expect("write");
    let out = repo.refused(&["accept", "--batch", &file]);
    assert!(out.contains("no one accepts on another principal's behalf"), "{out}");
}

#[test]
fn a_clone_signs_only_its_own_rows_on_its_own_branch() {
    let repo = governed();
    let ids = three(&repo);
    let mut file = ledger_core::batch_file::read(std::path::Path::new(&batch(&repo, &ids, None))).expect("read");
    file.rows.push(Row {
        repository: "elsewhere".into(),
        branch: Some("agent/7".into()),
        decision: format!("dec:{NS}/01K9YYYYYYYYYYYYYYYYYYYYYY"),
        version: format!("sha256:{}", "1".repeat(64)),
        grant: Some("grant:01K9YYYYYYYYYYYYYYYYYYYYYY".into()),
    });
    let path = repo.path().join("batch.yml");
    ledger_core::batch_file::write(&path, &file).expect("write");
    let p = path.display().to_string();
    let spans = repo.ledger(&["accept", "--batch", &p]);
    assert_ne!(spans.status.code(), Some(0));
    assert!(common::both(&spans).contains("--repository"), "{}", common::both(&spans));
    // This clone's rows only; the other repository's row is covered by the
    // manifest but signed in its own clone.
    let dry = repo.ok(&["accept", "--batch", &p, "--repository", "here"]);
    assert_eq!(dry.matches("under grant:").count(), 3, "{dry}");
    assert_eq!(manifest(&dry), ledger_core::batch_file::manifest(&{
        let mut resolved = file.clone();
        for r in resolved.rows.iter_mut().filter(|r| r.repository == "here") {
            r.grant = Some(hand::word(&dry, "grant:"));
        }
        resolved
    }), "the clone's manifest is the file's, with its own rows' grants");
    // A row on another branch than the one checked out refuses.
    for r in file.rows.iter_mut().filter(|r| r.repository == "here") {
        r.branch = Some("not-checked-out".into());
    }
    ledger_core::batch_file::write(&path, &file).expect("write");
    let wrong = repo.ledger(&["accept", "--batch", &p, "--repository", "here"]);
    assert_ne!(wrong.status.code(), Some(0));
    assert!(common::both(&wrong).contains("is checked out"), "{}", common::both(&wrong));
}

//! Store-loading cases: whole-store reporting, filing faults, format gates.

use crate::testkit;

use super::*;

struct Repo {
    dir: tempfile::TempDir,
}

impl Repo {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join(".decisions/ns").join(testkit::NS).join("sets")).expect("mkdir");
        std::fs::create_dir_all(dir.path().join(".decisions/ns").join(testkit::NS).join("log")).expect("mkdir");
        Self { dir }
    }

    fn path(&self) -> &Path {
        self.dir.path()
    }

    /// Write a file under the fixture namespace's directory.
    fn write(&self, relative: &str, body: &str) {
        let path = self.dir.path().join(".decisions/ns").join(testkit::NS).join(relative);
        std::fs::write(path, body).expect("write");
    }

    /// Write a file at a flat path of revision v1.8, directly under `.decisions/`.
    fn write_flat(&self, relative: &str, body: &str) {
        let path = self.dir.path().join(".decisions").join(relative);
        std::fs::create_dir_all(path.parent().expect("dir")).expect("mkdir");
        std::fs::write(path, body).expect("write");
    }

    fn write_set(&self) {
        let text = serde_yaml::to_string(&testkit::set()).expect("serialize");
        self.write("sets/ledger-design.yml", &text);
    }

    fn write_log(&self) {
        let cs = testkit::changeset(vec![testkit::sealed(testkit::version())], Vec::new());
        let text = serde_yaml::to_string(&cs).expect("serialize");
        self.write(&format!("log/{}.yml", testkit::CS_ULID), &text);
    }

    fn load(&self) -> Store {
        load(self.path())
    }
}

#[test]
fn a_well_formed_store_loads_with_no_faults() {
    let repo = Repo::new();
    repo.write_set();
    repo.write_log();
    let store = repo.load();
    assert!(store.schema_findings.is_empty(), "{:?}", store.schema_findings);
    assert_eq!(store.sets.len(), 1);
    assert_eq!(store.log.len(), 1);
    assert_eq!(store.entry_count(), 3, "one set, one decision, one version");
    assert!(store.set_in(testkit::NS, "ledger-design").is_some());
    assert_eq!(store.log[0].namespace, testkit::NS, "a file's namespace is its directory");
    assert_eq!(store.sets[0].namespace, testkit::NS);
}

/// AC-63: a store with any flat path at the verified commit is a schema fault
/// (LP-3.34, ruling 63) — each flat file named, the namespaced files still read.
#[test]
fn a_file_at_a_flat_path_is_a_schema_fault_and_names_the_path() {
    let repo = Repo::new();
    repo.write_set();
    repo.write_log();
    let text = serde_yaml::to_string(&testkit::set()).expect("serialize");
    repo.write_flat("sets/ledger-design.yml", &text);
    repo.write_flat("log/01K2C4YQJ3F8M0PT5W7NZ9RDXZ.yml", "format: 1\n");
    repo.write_flat("roles/steward.yml", "format: 6\n");
    repo.write_flat("sig/01K2C4YQJ3F8M0PT5W7NZ9RDXZ.ssh.sig", "x");
    repo.write_flat("allowed_signers", "# x\n");
    let store = repo.load();
    let faults: Vec<&str> = store.schema_findings.iter().map(|f| f.subject.as_str()).collect();
    assert_eq!(
        faults,
        [
            ".decisions/allowed_signers",
            ".decisions/log/01K2C4YQJ3F8M0PT5W7NZ9RDXZ.yml",
            ".decisions/roles/steward.yml",
            ".decisions/sets/ledger-design.yml",
            ".decisions/sig/01K2C4YQJ3F8M0PT5W7NZ9RDXZ.ssh.sig",
        ],
        "{:?}",
        store.schema_findings
    );
    assert!(store.schema_findings.iter().all(|f| f.message.contains("LP-3.34")), "{:?}", store.schema_findings);
    assert_eq!((store.sets.len(), store.log.len()), (1, 1), "the namespaced files are read; the flat ones are not");
}

/// A store with no `ns/` directory and a flat path is a schema fault too;
/// nothing of the flat layout is read.
#[test]
fn a_flat_only_store_is_refused_and_nothing_of_it_is_read() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repo = Repo { dir };
    std::fs::remove_dir_all(repo.path().join(".decisions")).ok();
    let text = serde_yaml::to_string(&testkit::set()).expect("serialize");
    repo.write_flat("sets/ledger-design.yml", &text);
    let cs = testkit::changeset(vec![testkit::sealed(testkit::version())], Vec::new());
    repo.write_flat(&format!("log/{}.yml", testkit::CS_ULID), &serde_yaml::to_string(&cs).expect("serialize"));
    let store = repo.load();
    assert!(store.sets.is_empty() && store.log.is_empty(), "the flat layout is not read");
    assert_eq!(store.schema_findings.len(), 2, "{:?}", store.schema_findings);
}

/// Two namespaces may each declare a set of one id (LP-3.36); a version
/// names the set of its own namespace (LP-5.22).
#[test]
fn a_set_id_is_unique_within_its_namespace_not_across_the_store() {
    let repo = Repo::new();
    repo.write_set();
    let text = serde_yaml::to_string(&testkit::set()).expect("serialize");
    let other = repo.path().join(".decisions/ns/hafeok.other/sets");
    std::fs::create_dir_all(&other).expect("mkdir");
    std::fs::write(other.join("ledger-design.yml"), &text).expect("write");
    let store = repo.load();
    assert!(store.schema_findings.is_empty(), "{:?}", store.schema_findings);
    assert_eq!(store.sets.len(), 2);
    assert!(store.set_in("hafeok.other", "ledger-design").is_some());
    assert!(store.set_in("hafeok.third", "ledger-design").is_none());
    repo.write("sets/ledger-design.yaml", &text);
    let twice = repo.load();
    assert!(twice.schema_findings.iter().any(|f| f.message.contains("declared twice in `hafeok.ledger`")), "{:?}", twice.schema_findings);
}

/// A file directly under `ns/`, or a directory there that is no namespace
/// name, is a schema fault: a namespace is a directory named by its id.
#[test]
fn a_stray_file_under_ns_and_a_misnamed_namespace_directory_are_faults() {
    let repo = Repo::new();
    repo.write_set();
    std::fs::write(repo.path().join(".decisions/ns/notes.txt"), "x").expect("write");
    std::fs::create_dir_all(repo.path().join(".decisions/ns/Not A Namespace/log")).expect("mkdir");
    let store = repo.load();
    let subjects: Vec<&str> = store.schema_findings.iter().map(|f| f.subject.as_str()).collect();
    assert!(subjects.contains(&".decisions/ns/notes.txt"), "{subjects:?}");
    assert!(subjects.contains(&"ns/Not A Namespace"), "{subjects:?}");
}

#[test]
fn an_absent_store_loads_empty_rather_than_failing() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = load(dir.path());
    assert!(store.sets.is_empty() && store.log.is_empty());
    assert!(store.schema_findings.is_empty());
}

#[test]
fn one_unparseable_file_does_not_stop_the_others() {
    let repo = Repo::new();
    repo.write_set();
    repo.write_log();
    repo.write("log/01K2C4YQJ3F8M0PT5W7NZ9RDXZ.yml", "format: 1\nid: not-a-change-set-id\n");
    let store = repo.load();
    assert_eq!(store.log.len(), 1, "the good file still loaded");
    assert_eq!(store.schema_findings.len(), 1);
    assert!(store.schema_findings[0].message.contains("does not parse"), "{:?}", store.schema_findings);
}

#[test]
fn a_file_whose_name_disagrees_with_its_id_is_a_fault() {
    let repo = Repo::new();
    repo.write_set();
    let cs = testkit::changeset(Vec::new(), Vec::new());
    let text = serde_yaml::to_string(&cs).expect("serialize");
    repo.write("log/01K2C4YQJ3F8M0PT5W7NZ9RDXZ.yml", &text);
    let store = repo.load();
    assert!(
        store.schema_findings.iter().any(|f| f.message.contains("but is filed as")),
        "{:?}",
        store.schema_findings
    );
}

#[test]
fn a_set_filed_under_the_wrong_name_is_a_fault() {
    let repo = Repo::new();
    let text = serde_yaml::to_string(&testkit::set()).expect("serialize");
    repo.write("sets/other-name.yml", &text);
    let store = repo.load();
    assert!(store.schema_findings.iter().any(|f| f.message.contains("but is filed as")));
}

#[test]
fn an_unknown_format_is_named_rather_than_assumed() {
    let repo = Repo::new();
    let mut set = testkit::set();
    set.format = 9;
    let text = serde_yaml::to_string(&set).expect("serialize");
    repo.write("sets/ledger-design.yml", &text);
    let store = repo.load();
    assert!(
        store.schema_findings.iter().any(|f| f.message.contains("declares format 9")),
        "{:?}",
        store.schema_findings
    );
}

#[test]
fn a_reserved_signature_in_a_log_file_is_caught_at_load() {
    let repo = Repo::new();
    repo.write_set();
    let sealed = testkit::sealed(testkit::version());
    let mut acceptance = testkit::acceptance(&sealed);
    acceptance.signature = "sig".into();
    let cs = testkit::changeset(vec![sealed], vec![acceptance]);
    let text = serde_yaml::to_string(&cs).expect("serialize");
    repo.write(&format!("log/{}.yml", testkit::CS_ULID), &text);
    let store = repo.load();
    assert!(store.schema_findings.iter().any(|f| f.message.contains("retired")));
}

#[test]
fn both_yaml_extensions_are_read() {
    let repo = Repo::new();
    let text = serde_yaml::to_string(&testkit::set()).expect("serialize");
    repo.write("sets/ledger-design.yaml", &text);
    assert_eq!(repo.load().sets.len(), 1);
}

#[test]
fn the_root_is_found_by_walking_up() {
    let repo = Repo::new();
    let nested = repo.path().join("a/b/c");
    std::fs::create_dir_all(&nested).expect("mkdir");
    assert_eq!(find_root(&nested).as_deref(), Some(repo.path()));
}

/// A change-set holding one sealed version with a reopen edge, declaring `format`.
fn write_reopening_log(repo: &Repo, format: u32) {
    let mut reopening = testkit::version();
    reopening.revisit_if = vec!["claim:DDD-x-01@sha256:abc".parse().expect("token")];
    let mut cs = testkit::changeset(vec![testkit::sealed(reopening)], Vec::new());
    cs.format = format;
    let text = serde_yaml::to_string(&cs).expect("serialize");
    repo.write(&format!("log/{}.yml", testkit::CS_ULID), &text);
}

#[test]
fn a_lower_format_change_set_carrying_revisit_if_is_a_fault() {
    for declared in 1..format::REVISIT_FORMAT {
        let repo = Repo::new();
        repo.write_set();
        write_reopening_log(&repo, declared);
        let store = repo.load();
        let faults: Vec<&str> = store.schema_findings.iter().map(|f| f.message.as_str()).collect();
        assert_eq!(faults, ["carries `revisit_if`, a format 4 field — declare `format: 4`"], "format {declared}");
    }
}

#[test]
fn a_format_4_change_set_carrying_revisit_if_parses() {
    let repo = Repo::new();
    repo.write_set();
    write_reopening_log(&repo, format::REVISIT_FORMAT);
    let store = repo.load();
    assert!(store.schema_findings.is_empty(), "{:?}", store.schema_findings);
    assert_eq!(store.log[0].file.versions[0].revisit_if.len(), 1);
}

/// A change-set holding a sealed version, its acceptance and a legacy-shape
/// revocation of that acceptance, declaring `format`.
fn write_legacy_revocation_log(repo: &Repo, format: u32) {
    let sealed = testkit::sealed(testkit::version());
    let acceptance = testkit::acceptance(&sealed);
    let mut cs = testkit::changeset(vec![sealed], vec![acceptance]);
    cs.revocations = vec![testkit::legacy_revocation("2026-08-11T09:00:00Z", "filed against the wrong version")];
    cs.format = format;
    let text = serde_yaml::to_string(&cs).expect("serialize");
    repo.write(&format!("log/{}.yml", testkit::CS_ULID), &text);
}

/// Format 6 retired the legacy revocation shape: declaring it, or anything
/// above it, over a file that uses the shape is a fault even though the
/// content alone needs format 1 (ruled 2026-10-05).
#[test]
fn a_legacy_revocation_in_a_file_declaring_format_6_or_above_is_a_fault() {
    for declared in [format::AUTHORITY_FORMAT, format::SIGNING_FORMAT] {
        let repo = Repo::new();
        repo.write_set();
        write_legacy_revocation_log(&repo, declared);
        let store = repo.load();
        assert!(
            store.schema_findings.iter().any(|f| f.message == "a format-6 revocation carries id, revokes, actor and hash"),
            "format {declared}: {:?}",
            store.schema_findings
        );
    }
    let repo = Repo::new();
    repo.write_set();
    write_legacy_revocation_log(&repo, format::KEY_FORMAT);
    assert!(repo.load().schema_findings.is_empty(), "format 5 still takes the legacy shape");
}

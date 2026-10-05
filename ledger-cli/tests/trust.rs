//! The trust root and the authority history (#70): signed key bindings and
//! D7, the position rule (D5 (c)), write-once roles (Q8), signed policy
//! changes (D1, D8), and the grant an act names (D9).

mod common;

use common::hand;
use common::Repo;
use ledger_core::authority::BindingAct;

const OWNER: &str = "owner@customer.example";
const ARCHITECT: &str = "architect@customer.example";
const NS: &str = "fixture.ledger";

fn governed() -> (Repo, String) {
    let repo = Repo::with_identity(OWNER);
    repo.declare();
    repo.ok(&["init", "--namespace", NS, "--external-ref", "contract 2026/117", "--without-key"]);
    let key = repo.bind_own_key(NS, "owner");
    hand::commit(&repo, "governed");
    (repo, key)
}

fn verify(repo: &Repo) -> (i32, String) {
    let out = repo.ledger(&["verify", "--no-blame"]);
    (out.status.code().unwrap_or(-1), common::both(&out))
}

fn genesis(repo: &Repo) -> String {
    let store = ledger_core::store::load(repo.path());
    store.log.iter().flat_map(|l| l.file.grants.iter()).find(|g| g.genesis).map(|g| g.id.to_string()).expect("genesis")
}

fn binding_of(repo: &Repo, who: &str) -> String {
    let store = ledger_core::store::load(repo.path());
    store
        .log
        .iter()
        .flat_map(|l| l.file.key_bindings.iter())
        .filter(|b| b.principal.as_str() == who && b.act.opens())
        .map(|b| b.id.to_string())
        .next_back()
        .expect("binding")
}

fn signers(repo: &Repo) -> String {
    std::fs::read_to_string(repo.path().join(".decisions/allowed_signers")).unwrap_or_default()
}

#[test]
fn an_unsigned_binding_for_an_existing_holder_fails_and_never_reaches_allowed_signers() {
    let (repo, _) = governed();
    let extra = repo.keygen("extra");
    let b = hand::binding(BindingAct::Add, OWNER, OWNER, NS, Some(&format!("{extra}.pub")), None, None);
    let id = b.id.to_string();
    hand::file(&repo, vec![b], Vec::new(), &[]);
    let (code, text) = verify(&repo);
    assert_eq!(code, 1);
    assert!(text.contains("[L011]") && text.contains(&id) && text.contains("not trusted"), "{text}");
    let derived = ledger_core::authority::signers::derive(&ledger_core::store::load(repo.path())).unwrap_or_default();
    assert_eq!(derived.lines().filter(|l| !l.starts_with('#')).count(), 1, "only the genesis key: {derived}");
}

#[test]
fn a_binding_signed_by_a_key_that_is_not_live_fails() {
    let (repo, owner) = governed();
    let old = binding_of(&repo, OWNER);
    let next = repo.keygen("next");
    repo.ok(&["identity", "rotate", &old, "--key-file", &format!("{next}.pub")]);
    hand::commit(&repo, "rotated");
    // A further key, signed by the key the rotation closed.
    let extra = repo.keygen("extra");
    let b = hand::binding(BindingAct::Add, OWNER, OWNER, NS, Some(&format!("{extra}.pub")), None, None);
    let ulid = b.id.ulid().to_string();
    hand::file(&repo, vec![b], Vec::new(), &[(&ulid, &owner)]);
    let (code, text) = verify(&repo);
    assert_eq!(code, 1);
    assert!(text.contains("[L011]") && text.contains("not trusted"), "{text}");
}

#[test]
fn d7_each_binding_act_by_an_allowed_and_a_disallowed_signer() {
    let (repo, owner) = governed();
    let g = genesis(&repo);
    // add: a principal's own first key is refused; the genesis holder's vouching is not.
    let arch = repo.keygen("architect");
    repo.act_as(ARCHITECT);
    repo.use_key(&arch);
    let refused = repo.refused(&["identity", "add", "--namespace", NS, "--key-file", &format!("{arch}.pub")]);
    assert!(refused.contains("first key is filed by the genesis holder"), "{refused}");
    repo.act_as(OWNER);
    repo.use_key(&owner);
    repo.ok(&["identity", "add", "--namespace", NS, "--for", ARCHITECT, "--key-file", &format!("{arch}.pub")]);
    let arch_binding = binding_of(&repo, ARCHITECT);
    assert!(signers(&repo).contains(ARCHITECT));
    // rotate: never the genesis holder's.
    let next = repo.keygen("architect-next");
    let refused = repo.refused(&["identity", "rotate", &arch_binding, "--key-file", &format!("{next}.pub")]);
    assert!(refused.contains("may not rotate"), "{refused}");
    repo.act_as(ARCHITECT);
    repo.use_key(&arch);
    repo.ok(&["identity", "rotate", &arch_binding, "--key-file", &format!("{next}.pub")]);
    let rotated = binding_of(&repo, ARCHITECT);
    // revoke: the principal's, or the genesis holder's — not a third party's.
    let third = "third@customer.example";
    let third_key = { repo.act_as(OWNER); repo.use_key(&owner); repo.vouch_for(NS, third, "third") };
    repo.act_as(third);
    repo.use_key(&third_key);
    let refused = repo.refused(&["identity", "revoke", &rotated]);
    assert!(refused.contains("may not revoke"), "{refused}");
    repo.act_as(OWNER);
    repo.use_key(&owner);
    repo.ok(&["identity", "revoke", &rotated]);
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "{text}");
    // A hand-written first binding for another principal, filed by a third party.
    let forged_key = repo.keygen("forged");
    let mut forged = hand::binding(BindingAct::Add, "fourth@customer.example", third, NS, Some(&format!("{forged_key}.pub")), None, Some(&g));
    forged.hash = ledger_core::authority::payload::binding_hash(&forged);
    let ulid = forged.id.ulid().to_string();
    hand::file(&repo, vec![forged], Vec::new(), &[(&ulid, &third_key)]);
    let (code, text) = verify(&repo);
    assert_eq!(code, 1);
    assert!(text.contains("D7") && text.contains("only the genesis holder files another's first key"), "{text}");
}

#[test]
fn a_policy_change_is_signed_under_the_policy_it_replaces() {
    let (repo, _) = governed();
    let out = repo.ok(&["policy", "set", "--namespace", NS, "--reaccept-within-days", "30"]);
    assert!(out.contains("replaces pol:"), "{out}");
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "{text}");
    let store = ledger_core::store::load(repo.path());
    let current = ledger_core::authority::Authority::build(&store).policy(NS).cloned().expect("policy");
    let mut next = current.clone();
    next.id = ledger_core::mint::UlidMint::system().mint_id("pol").expect("id");
    next.replaces = Some(current.hash.clone());
    next.require_sk = true;
    next.at = chrono::Utc::now();
    next.hash = ledger_core::authority::payload::policy_hash(&next);
    let id = next.id.to_string();
    hand::file(&repo, Vec::new(), vec![next], &[]);
    let (code, text) = verify(&repo);
    assert_eq!(code, 1);
    assert!(text.contains("[L011]") && text.contains(&id), "an unsigned policy change: {text}");
}

#[test]
fn acts_before_the_first_policy_stand_and_acts_after_it_are_checked() {
    let repo = Repo::with_identity(ARCHITECT);
    repo.declare();
    let early = repo.add("Money is decimal.", &[]);
    repo.ok_tty(&["accept", &early]);
    hand::commit(&repo, "pre-policy acceptance");
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "{text}");
    assert!(text.contains(&format!("namespace `{NS}` has no policy")), "the unchecked notice: {text}");
    repo.act_as(OWNER);
    repo.ok(&["init", "--namespace", NS, "--external-ref", "contract 2026/117", "--without-key"]);
    let owner = repo.bind_own_key(NS, "owner");
    let arch = repo.vouch_for(NS, ARCHITECT, "architect");
    hand::commit(&repo, "opted in");
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "the earlier acceptance stands as a pre-policy act: {text}");
    assert!(!text.contains("has no policy"), "{text}");
    let late = repo.add("Time is UTC.", &[]);
    let at = chrono::Utc::now() + chrono::Duration::seconds(1);
    hand::accept(&repo, &hand::HandAccept { decision: &late, actor: ARCHITECT, at, under: None, key: Some(&arch) });
    hand::commit(&repo, "ungranted acceptance after the policy");
    let (code, text) = verify(&repo);
    assert_eq!(code, 1);
    assert!(text.contains("[A006]"), "{text}");
    let _ = owner;
}

#[test]
fn removing_a_policy_fails_l007_because_opting_in_is_one_way() {
    let (repo, _) = governed();
    let store = ledger_core::store::load(repo.path());
    let file = store.log.iter().find(|l| !l.file.policies.is_empty()).map(|l| l.path.clone()).expect("policy file");
    std::fs::remove_file(&file).expect("remove");
    hand::commit(&repo, "dropped the policy");
    let (code, text) = verify(&repo);
    assert_eq!(code, 1);
    assert!(text.contains("[L007]") && text.contains("one-way"), "{text}");
}

#[test]
fn a_role_file_edited_after_landing_fails_l007() {
    let (repo, _) = governed();
    let path = repo.path().join(".decisions/roles/acceptor.yml");
    let text = std::fs::read_to_string(&path).expect("role");
    std::fs::write(&path, text.replace("- accept-decision", "- accept-decision\n- grant-role")).expect("edit");
    let (code, out) = verify(&repo);
    assert_eq!(code, 1, "uncommitted: {out}");
    assert!(out.contains("[L007]") && out.contains("write-once"), "{out}");
    hand::commit(&repo, "edited a role");
    let (code, out) = verify(&repo);
    assert_eq!(code, 1, "committed: {out}");
    assert!(out.contains("roles/acceptor.yml"), "{out}");
}

#[test]
fn with_two_qualifying_grants_as_is_required_and_the_narrowest_scope_wins() {
    let (repo, _) = governed();
    let mut grants = Vec::new();
    for scope in [format!("ns:{NS}"), "set:ledger-design".to_string()] {
        let grant = hand::word(&repo.ok(&["grant", "new", "acceptor", "--to", OWNER, "--scope", &scope]), "grant:");
        repo.ok(&["grant", "accept", &grant]);
        grants.push(grant);
    }
    let id = repo.add("Money is decimal.", &[]);
    let ambiguous = repo.refused_tty(&["accept", &id]);
    assert!(ambiguous.contains("--as <role>"), "{ambiguous}");
    let ok = repo.ok_tty(&["accept", &id, "--as", "acceptor"]);
    assert!(ok.contains(&format!("under {} (`acceptor`)", grants[1])), "the set grant is the narrowest: {ok}");
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "{text}");
}

#[test]
fn the_escalation_guard_decides_candidacy_for_grant_new() {
    let (repo, _) = governed();
    repo.ok(&["role", "declare", "delegate", "--may", "grant-role"]);
    repo.ok(&["role", "declare", "lead", "--may", "grant-role", "--may", "accept-decision"]);
    let scope = format!("ns:{NS}");
    // The genesis holder also holds a one-capability granting role.
    let own = hand::word(&repo.ok(&["grant", "new", "delegate", "--to", OWNER, "--scope", &scope]), "grant:");
    repo.ok(&["grant", "accept", &own]);
    // Granting `acceptor`: `delegate` grants only `delegate`, so the genesis
    // is the one candidate, and it is not refused for claiming more.
    let out = repo.ok(&["grant", "new", "acceptor", "--to", ARCHITECT, "--scope", &scope]);
    assert!(out.contains(&format!("under {}", genesis(&repo))), "{out}");
    // Granting `delegate`: both qualify; `--as` is required, and the genesis
    // role — a superset of `delegate` — is refused for the narrower one.
    let third = "third@customer.example";
    let ambiguous = repo.refused(&["grant", "new", "delegate", "--to", third, "--scope", &scope]);
    assert!(ambiguous.contains("--as <role>"), "{ambiguous}");
    let broader = repo.refused(&["grant", "new", "delegate", "--to", third, "--scope", &scope, "--as", "steward"]);
    assert!(broader.contains("claims a subset of it"), "the broader role is refused: {broader}");
    let out = repo.ok(&["grant", "new", "delegate", "--to", third, "--scope", &scope, "--as", "delegate"]);
    assert!(out.contains(&format!("under {own} (`delegate`)")), "{out}");
    let thirds = hand::word(&out, "grant:");
    let out = repo.ok_tty(&["grant", "revoke", &thirds, "--reason", "not needed"]);
    assert!(out.contains(&format!("under {} (`steward`)", genesis(&repo))), "grant revoke shows its grant: {out}");
    // Below the genesis: a holder of `lead` and `delegate` grants each role
    // under its own grant, and no other.
    let arch = repo.vouch_for(NS, ARCHITECT, "architect");
    let mut held = Vec::new();
    for (role, as_role) in [("lead", "steward"), ("delegate", "delegate")] {
        let out = repo.ok(&["grant", "new", role, "--to", ARCHITECT, "--scope", &scope, "--as", as_role]);
        held.push(hand::word(&out, "grant:"));
    }
    repo.act_as(ARCHITECT);
    repo.use_key(&arch);
    for g in &held {
        repo.ok(&["grant", "accept", g]);
    }
    let out = repo.ok(&["grant", "new", "delegate", "--to", third, "--scope", &scope]);
    assert!(out.contains(&format!("under {} (`delegate`)", held[1])), "the one candidate, no `--as`: {out}");
    let refused = repo.refused(&["grant", "new", "acceptor", "--to", third, "--scope", &scope]);
    assert!(refused.contains("may grant only that role"), "{refused}");
}

#[test]
fn a_policy_change_in_a_format_6_file_is_a_schema_fault() {
    let (repo, _) = governed();
    let store = ledger_core::store::load(repo.path());
    let current = ledger_core::authority::Authority::build(&store).policy(NS).cloned().expect("policy");
    let mut next = current.clone();
    next.id = ledger_core::mint::UlidMint::system().mint_id("pol").expect("id");
    next.replaces = Some(current.hash.clone());
    next.at = chrono::Utc::now();
    next.hash = ledger_core::authority::payload::policy_hash(&next);
    let id = next.id.to_string();
    hand::file(&repo, Vec::new(), vec![next], &[]);
    let file = hand::file_holding(&repo, &id);
    let text = std::fs::read_to_string(&file).expect("read");
    std::fs::write(&file, text.replace("format: 7", "format: 6")).expect("write");
    let (code, text) = verify(&repo);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains("[SCHEMA]") && text.contains("carries a namespace policy, a format 7 entry"), "{text}");
}

const SECOND: &str = "second.ledger";

#[test]
fn the_genesis_holder_self_binds_once_per_store_and_signs_later_namespaces_with_a_trusted_key() {
    let (repo, _) = governed();
    repo.ok(&["init", "--namespace", SECOND, "--external-ref", "contract 2026/117"]);
    // Their first key in the second namespace: their own add, signed by the
    // key already trusted in the first.
    let next = repo.keygen("owner-second");
    let out = repo.ok(&["identity", "add", "--namespace", SECOND, "--key-file", &format!("{next}.pub")]);
    assert!(out.contains(&format!("in `{SECOND}`")), "{out}");
    let store = ledger_core::store::load(repo.path());
    let second = store.log.iter().flat_map(|l| l.file.key_bindings.iter()).find(|b| b.namespace == SECOND).expect("binding");
    assert!(!second.self_bound && second.mandate.is_none(), "not self-bound: {second:?}");
    hand::commit(&repo, "second namespace");
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "{text}");
}

#[test]
fn a_hand_written_self_bound_binding_in_a_second_namespace_fails() {
    let (repo, _) = governed();
    repo.ok(&["init", "--namespace", SECOND, "--external-ref", "contract 2026/117"]);
    hand::commit(&repo, "second namespace opted in");
    let forged_key = repo.keygen("owner-forged");
    let mut forged = hand::binding(BindingAct::Add, OWNER, OWNER, SECOND, Some(&format!("{forged_key}.pub")), None, None);
    forged.self_bound = true;
    forged.mandate = Some("contract 2026/117".into());
    forged.hash = ledger_core::authority::payload::binding_hash(&forged);
    let id = forged.id.to_string();
    let ulid = forged.id.ulid().to_string();
    hand::file(&repo, vec![forged], Vec::new(), &[(&ulid, &forged_key)]);
    let (code, text) = verify(&repo);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains(&id) && text.contains("D7") && text.contains("first in the store"), "{text}");
}

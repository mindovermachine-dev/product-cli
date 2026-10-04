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
    repo.ok(&["init", "--namespace", NS, "--external-ref", "contract 2026/117"]);
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
    next.at_hashed = true;
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
    repo.ok(&["init", "--namespace", NS, "--external-ref", "contract 2026/117"]);
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
fn the_escalation_guard_compares_the_role_the_act_names() {
    let (repo, _) = governed();
    repo.ok(&["role", "declare", "delegate", "--may", "grant-role"]);
    repo.ok(&["role", "declare", "lead", "--may", "grant-role", "--may", "accept-decision"]);
    let arch = repo.vouch_for(NS, ARCHITECT, "architect");
    let mut grants = Vec::new();
    for role in ["lead", "delegate"] {
        grants.push(hand::word(&repo.ok(&["grant", "new", role, "--to", ARCHITECT, "--scope", &format!("ns:{NS}")]), "grant:"));
    }
    repo.act_as(ARCHITECT);
    repo.use_key(&arch);
    for g in &grants {
        repo.ok(&["grant", "accept", g]);
    }
    let third = "third@customer.example";
    let ambiguous = repo.refused(&["grant", "new", "delegate", "--to", third, "--scope", &format!("ns:{NS}")]);
    assert!(ambiguous.contains("--as <role>"), "{ambiguous}");
    let broader = repo.refused(&["grant", "new", "delegate", "--to", third, "--scope", &format!("ns:{NS}"), "--as", "lead"]);
    assert!(broader.contains("fewer claims"), "the broader role is refused: {broader}");
    // As `delegate`, granting `delegate` is the role the act names.
    repo.ok(&["grant", "new", "delegate", "--to", third, "--scope", &format!("ns:{NS}"), "--as", "delegate"]);
    let refused = repo.refused(&["grant", "new", "lead", "--to", third, "--scope", &format!("ns:{NS}"), "--as", "delegate"]);
    assert!(refused.contains("may grant only that role"), "{refused}");
}

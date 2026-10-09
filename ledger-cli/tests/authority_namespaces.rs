//! Authority is per namespace (LP-6.31, ruling 47; PRD §3.2; issue 5).
//!
//! Each namespace is opened with its own genesis grant, roles and first
//! policy, declared and filed under its own directory; a scope is read
//! inside the grant's namespace; `A003` and `A005` count per namespace
//! because the graph stage runs over each namespace's graph alone; and a
//! store that shared one genesis across namespaces, as the writer built one
//! before this, fails verification (AC-47 first bullet, AC-48, AC-68).
//!
//! Keys are still bound and trusted store-wide here (issue 6 scopes them),
//! so the genesis holder's key bound in A signs in B once `init` binds it
//! there.

mod common;

use common::{hand, Repo};
use ledger_core::authority::payload::grant_hash;
use ledger_core::authority::{Grant, GrantScope, Order, Policy, Scheme};
use ledger_core::changeset::ChangeSet;
use ledger_core::mint::UlidMint;

const OWNER: &str = "owner@customer.example";
const ARCHITECT: &str = "architect@customer.example";
const MANDATE_A: &str = "contract 2026/117";
const MANDATE_B: &str = "contract 2026/118";
const A: &str = "a.ledger";
const B: &str = "b.ledger";

fn verify(repo: &Repo) -> (i32, String) {
    let out = repo.ledger(&["verify", "--no-blame"]);
    (out.status.code().unwrap_or(-1), common::both(&out))
}

fn store(repo: &Repo) -> ledger_core::store::Store {
    ledger_core::store::load(repo.path())
}

/// The live genesis grants, by the namespace whose directory holds them.
fn genesis_by_namespace(repo: &Repo) -> Vec<(String, String)> {
    let store = store(repo);
    let mut out: Vec<(String, String)> = store
        .log
        .iter()
        .flat_map(|l| l.file.grants.iter().filter(|g| g.genesis).map(move |g| (l.namespace.clone(), g.id.to_string())))
        .collect();
    out.sort();
    out
}

fn decision(repo: &Repo, ns: &str, statement: &str) -> String {
    let out = repo.ok(&["add", "--set", "ledger-design", "--namespace", ns, "--statement", statement, "--store", "constraint", "--discharge", "analyzer:DEC001"]);
    common::decision_id(&out)
}

/// Two namespaces, each opened on its own mandate by the same holder, a set
/// declared in each, committed.
fn two_governed() -> (Repo, String) {
    let repo = Repo::with_identity(OWNER);
    let key = repo.keygen("owner");
    repo.use_key(&key);
    repo.ok(&["init", "--namespace", A, "--external-ref", MANDATE_A]);
    repo.ok(&["init", "--namespace", B, "--external-ref", MANDATE_B]);
    repo.declare_in(A);
    repo.declare_in(B);
    hand::commit(&repo, "two namespaces, two trust roots");
    (repo, key)
}

/// An acceptor grant to `who` over `scope`, filed in `ns` and accepted.
fn acceptor(repo: &Repo, ns: &str, who: &str, scope: &str) -> String {
    let grant = hand::word(&repo.ok(&["grant", "new", "acceptor", "--to", who, "--scope", scope, "--namespace", ns]), "grant:");
    repo.act_as(who);
    repo.ok(&["grant", "accept", &grant]);
    repo.act_as(OWNER);
    grant
}

/// File a grant by hand under `dir`'s directory, past the verb.
fn file_grant(repo: &Repo, dir: &str, grant: Grant) -> String {
    let id = grant.id.to_string();
    let mut mint = UlidMint::system();
    let mut cs = ChangeSet::empty(ledger_core::format::SIGNING_FORMAT, mint.mint_id("cs").expect("cs id"), grant.at, OWNER.parse().expect("identity"), None);
    cs.grants.push(grant);
    let log = repo.log_dir(dir);
    std::fs::create_dir_all(&log).expect("log dir");
    std::fs::write(log.join(cs.file_name()), serde_yaml::to_string(&cs).expect("yaml")).expect("write");
    id
}

// ---- AC-47, first bullet -------------------------------------------------

#[test]
fn each_namespace_has_its_own_genesis_roles_and_policy_and_a003_a005_do_not_fire() {
    let (repo, _) = two_governed();
    let genesis = genesis_by_namespace(&repo);
    assert_eq!(genesis.len(), 2, "one trust root per namespace: {genesis:?}");
    assert_eq!(genesis[0].0, A);
    assert_eq!(genesis[1].0, B);
    for ns in [A, B] {
        assert!(repo.roles_dir(ns).join("steward.yml").is_file(), "`{ns}` declares its own root role");
        assert!(repo.roles_dir(ns).join("acceptor.yml").is_file(), "`{ns}` declares its own accept role");
        let store = store(&repo);
        let policy = store.log.iter().flat_map(|l| l.file.policies.iter()).find(|p| p.namespace == ns).expect("policy");
        let under = policy.under.as_ref().map(ToString::to_string).expect("under");
        assert!(genesis.contains(&(ns.to_string(), under)), "`{ns}`'s first policy is under its own genesis");
    }
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "{text}");
    assert!(!text.contains("[A005]") && !text.contains("[A003]"), "{text}");
    assert!(text.contains("conformant"), "{text}");
}

#[test]
fn a_second_init_of_a_namespace_needs_its_own_mandate_and_a_namespace_has_one_trust_root() {
    let repo = Repo::with_identity(OWNER);
    repo.ok(&["init", "--namespace", A, "--external-ref", MANDATE_A, "--without-key"]);
    // Every namespace is opened on its own mandate (ruling 47): no joining
    // the first namespace's genesis.
    let out = repo.ledger(&["init", "--namespace", B]);
    assert_eq!(out.status.code(), Some(2), "{}", common::both(&out));
    assert!(common::both(&out).contains("--external-ref"), "{}", common::both(&out));
    assert!(genesis_by_namespace(&repo).len() == 1, "nothing written for `{B}`");
    // And a namespace is opened once.
    let again = repo.ledger(&["init", "--namespace", A, "--external-ref", "another"]);
    assert_eq!(again.status.code(), Some(1), "{}", common::both(&again));
    assert!(common::both(&again).contains("already under policy"), "{}", common::both(&again));
}

#[test]
fn a_grant_over_star_in_a_covers_a_alone() {
    let (repo, key) = two_governed();
    // `*` is the whole of the grant's own namespace (ruling 67). With two
    // namespaces the grant names where it is filed.
    let refused = repo.ledger(&["grant", "new", "acceptor", "--to", ARCHITECT, "--scope", "*"]);
    assert_eq!(refused.status.code(), Some(2), "{}", common::both(&refused));
    assert!(common::both(&refused).contains("pass --namespace"), "{}", common::both(&refused));
    acceptor(&repo, A, ARCHITECT, "*");
    let in_a = decision(&repo, A, "Accepted in A under a star grant of A.");
    let in_b = decision(&repo, B, "Not acceptable in B under A's star.");
    hand::commit(&repo, "grants and decisions");
    let arch = repo.keygen("architect");
    repo.ok(&["identity", "add", "--namespace", A, "--key-file", &format!("{arch}.pub"), "--for", ARCHITECT]);
    repo.act_as(ARCHITECT);
    repo.use_key(&arch);
    repo.ok_tty(&["accept", &in_a]);
    let refused = repo.refused_tty(&["accept", &in_b]);
    assert!(refused.contains(&format!("may not accept in `{B}`")) && refused.contains("holds no grant"), "{refused}");
    repo.act_as(OWNER);
    repo.use_key(&key);
    hand::commit(&repo, "accepted in A only");
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "{text}");
}

#[test]
fn a_grant_scoped_to_another_namespace_is_refused_by_the_verb_and_a_schema_fault_by_hand() {
    let (repo, _) = two_governed();
    let refused = repo.ledger(&["grant", "new", "acceptor", "--to", ARCHITECT, "--scope", &format!("ns:{B}"), "--namespace", A]);
    assert_eq!(refused.status.code(), Some(2), "{}", common::both(&refused));
    assert!(common::both(&refused).contains("names another namespace"), "{}", common::both(&refused));
    // Past the verb: a grant filed under A scoped `ns:b.ledger` could never
    // have effect (LP-6.16).
    let genesis = genesis_by_namespace(&repo).into_iter().find(|(ns, _)| ns == A).map(|(_, id)| id).expect("A's genesis");
    let mut grant = Grant {
        id: UlidMint::system().mint_id("grant").expect("id"),
        role: "acceptor".into(),
        scope: GrantScope::Namespace(B.into()),
        holder: ARCHITECT.parse().expect("identity"),
        granted_by: OWNER.parse().expect("identity"),
        order: Order::PRIMARY,
        limits: Vec::new(),
        genesis: false,
        external_ref: None,
        supersedes: None,
        under: Some(genesis.parse().expect("grant id")),
        at: chrono::Utc::now(),
        hash: ledger_core::hash::VersionHash::zero(),
    };
    grant.hash = grant_hash(&grant);
    let id = file_grant(&repo, A, grant);
    hand::commit(&repo, "a grant scoped elsewhere, by hand");
    let (code, text) = verify(&repo);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains(&format!("[SCHEMA] {id}")) && text.contains(&format!("scoped `ns:{B}` but filed under `{A}`")), "{text}");
}

// ---- AC-48 ---------------------------------------------------------------

#[test]
fn no_grant_carries_a_namespace_field_and_authority_records_sit_apart_from_decisions() {
    let (repo, _) = two_governed();
    acceptor(&repo, A, OWNER, &format!("ns:{A}"));
    decision(&repo, A, "A decision beside the authority records.");
    let store = store(&repo);
    for logged in &store.log {
        let cs = &logged.file;
        for g in &cs.grants {
            let text = serde_yaml::to_string(g).expect("yaml");
            assert!(!text.contains("namespace"), "a grant belongs by directory, never by a field (PRD §3.12): {text}");
        }
        let authority = !cs.grants.is_empty() || !cs.grant_acceptances.is_empty() || !cs.policies.is_empty() || !cs.key_bindings.is_empty() || !cs.revocations.is_empty();
        if authority {
            assert!(cs.decisions.is_empty() && cs.versions.is_empty() && cs.acceptances.is_empty(), "authority records stay in change-sets of their own: {}", logged.path.display());
        }
    }
}

// ---- AC-68 ---------------------------------------------------------------

/// B's first policy as the writer filed it before this: under A's genesis,
/// signed by the holder's key trusted in A, with the holder's own `add`
/// in B signed the same way.
#[test]
fn a_store_that_shared_one_genesis_across_namespaces_fails_a006_and_d7() {
    let repo = Repo::with_identity(OWNER);
    let key = repo.keygen("owner");
    repo.use_key(&key);
    repo.ok(&["init", "--namespace", A, "--external-ref", MANDATE_A]);
    hand::commit(&repo, "A");
    let store = store(&repo);
    let genesis = store.log.iter().flat_map(|l| l.file.grants.iter()).find(|g| g.genesis).cloned().expect("A's genesis");
    let mut policy = Policy {
        id: UlidMint::system().mint_id("pol").expect("id"),
        namespace: B.into(),
        schemes: vec![Scheme::Ssh],
        require_sk: false,
        accept_role: "acceptor".into(),
        reaccept_within_days: None,
        replaces: None,
        by: OWNER.parse().expect("identity"),
        under: Some(genesis.id.clone()),
        at: chrono::Utc::now(),
        hash: ledger_core::hash::VersionHash::zero(),
    };
    policy.hash = ledger_core::authority::payload::policy_hash(&policy);
    let mut binding = hand::binding(ledger_core::authority::BindingAct::Add, OWNER, OWNER, B, Some(&format!("{key}.pub")), None, None);
    binding.at = policy.at;
    binding.hash = ledger_core::authority::payload::binding_hash(&binding);
    let (pol_id, pol_ulid) = (policy.id.to_string(), policy.id.ulid().to_string());
    let (key_id, key_ulid) = (binding.id.to_string(), binding.id.ulid().to_string());
    hand::file(&repo, vec![binding], vec![policy], &[(&pol_ulid, &key), (&key_ulid, &key)]);
    hand::commit(&repo, "B under A's genesis, as the writer once filed it");
    let (code, text) = verify(&repo);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains(&format!("[A006] {pol_id}")) && text.contains(&format!("no live genesis grant of `{B}`")), "{text}");
    assert!(text.contains(&format!("[SCHEMA] {key_id}")) && text.contains("D7"), "its first key binding leans on no other namespace: {text}");
}

// ---- Carried: no agent, no non-interactive acceptance, in a later namespace

#[test]
fn no_agent_identity_and_no_non_interactive_session_produces_an_acceptance_in_a_later_namespace() {
    let (repo, _) = two_governed();
    acceptor(&repo, B, OWNER, &format!("ns:{B}"));
    let id = decision(&repo, B, "Decided in B.");
    let before = repo.log_files();
    let piped = repo.piped(&["accept", &id]);
    assert_ne!(piped.status.code(), Some(0), "{}", common::both(&piped));
    for agent in ["claude@anthropic.com", "noreply@anthropic.com", "github-actions@github.com"] {
        repo.act_as(agent);
        let out = repo.tty(&["accept", &id]);
        assert_ne!(out.status.code(), Some(0), "{agent}: {}", common::both(&out));
    }
    assert_eq!(repo.log_files(), before, "nothing filed");
    repo.act_as(OWNER);
    repo.ok_tty(&["accept", &id]);
    assert_eq!(repo.log_files().len(), before.len() + 1, "the holder, at a terminal, accepts in B");
}

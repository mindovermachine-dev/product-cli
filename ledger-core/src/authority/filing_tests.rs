//! D7: each binding act, by a signer the rule allows or refuses.

use crate::authority::fixture::{self, accepted, genesis, role};
use crate::authority::{BindingAct, Capability, KeyBinding};
use crate::hash::VersionHash;
use crate::testkit;

use super::*;

const OWNER: &str = fixture::GENESIS_HOLDER;
const ARCH: &str = "architect@customer.example";
const NS: &str = "hafeok.ledger";

fn bind(tail: &str, act: BindingAct, principal: &str, by: &str) -> KeyBinding {
    KeyBinding {
        id: format!("key:{}", fixture::ulid(tail)).parse().expect("id"),
        act,
        principal: testkit::identity(principal),
        namespace: NS.into(),
        key_type: act.opens().then(|| "ssh-ed25519".into()),
        key: act.opens().then(|| format!("AAAA{tail}")),
        closes: None,
        self_bound: false,
        mandate: None,
        by: testkit::identity(by),
        under: None,
        at: testkit::stamp("2026-10-03T00:00:00Z"),
        hash: VersionHash::zero(),
    }
}

fn store(bindings: Vec<KeyBinding>) -> crate::store::Store {
    let g = genesis("1", "steward");
    let mut cs = fixture::changeset(vec![g.clone()], vec![accepted("2", &g)]);
    cs.key_bindings = bindings;
    fixture::store(vec![role("steward", Capability::ROOT)], cs)
}

fn genesis_id() -> crate::id::GrantId {
    genesis("1", "steward").id
}

fn self_bound() -> KeyBinding {
    let mut b = bind("10", BindingAct::Add, OWNER, OWNER);
    b.self_bound = true;
    b.mandate = Some("contract 2026/117".into());
    b
}

fn judge(prior: Vec<KeyBinding>, b: &KeyBinding) -> Result<(), String> {
    let store = store(prior);
    may_file(&Authority::build(&store), b)
}

#[test]
fn the_genesis_self_bound_binding_is_the_first_in_its_namespace() {
    assert_eq!(judge(Vec::new(), &self_bound()), Ok(()));
    let mut not_genesis = self_bound();
    not_genesis.principal = testkit::identity(ARCH);
    not_genesis.by = testkit::identity(ARCH);
    assert!(judge(Vec::new(), &not_genesis).is_err());
}

#[test]
fn a_principals_first_key_is_filed_by_the_genesis_holder_under_the_genesis_grant() {
    let own = bind("11", BindingAct::Add, ARCH, ARCH);
    assert!(judge(vec![self_bound()], &own).expect_err("own first key").contains("D7"));
    let mut vouched = bind("11", BindingAct::Add, ARCH, OWNER);
    assert!(judge(vec![self_bound()], &vouched).is_err(), "without `under`");
    vouched.under = Some(genesis_id());
    assert_eq!(judge(vec![self_bound()], &vouched), Ok(()));
    let mut stranger = bind("11", BindingAct::Add, ARCH, "third@customer.example");
    stranger.under = Some(genesis_id());
    assert!(judge(vec![self_bound()], &stranger).is_err());
}

fn architect_key() -> KeyBinding {
    let mut b = bind("11", BindingAct::Add, ARCH, OWNER);
    b.under = Some(genesis_id());
    b
}

#[test]
fn a_further_add_and_a_rotate_are_the_principals_own() {
    let prior = vec![self_bound(), architect_key()];
    assert_eq!(judge(prior.clone(), &bind("12", BindingAct::Add, ARCH, ARCH)), Ok(()));
    let mut again = bind("12", BindingAct::Add, ARCH, OWNER);
    again.under = Some(genesis_id());
    assert!(judge(prior.clone(), &again).is_err(), "the genesis holder files only a first key");
    let mut rotate = bind("13", BindingAct::Rotate, ARCH, ARCH);
    rotate.closes = Some(architect_key().id);
    assert_eq!(judge(prior.clone(), &rotate), Ok(()));
    let mut by_owner = bind("13", BindingAct::Rotate, ARCH, OWNER);
    by_owner.closes = Some(architect_key().id);
    by_owner.under = Some(genesis_id());
    assert!(judge(prior, &by_owner).is_err(), "rotation is never the genesis holder's");
}

#[test]
fn a_revoke_is_the_principals_or_the_genesis_holders() {
    let prior = vec![self_bound(), architect_key()];
    let mut own = bind("14", BindingAct::Revoke, ARCH, ARCH);
    own.closes = Some(architect_key().id);
    assert_eq!(judge(prior.clone(), &own), Ok(()));
    let mut by_genesis = bind("14", BindingAct::Revoke, ARCH, OWNER);
    by_genesis.closes = Some(architect_key().id);
    by_genesis.under = Some(genesis_id());
    assert_eq!(judge(prior.clone(), &by_genesis), Ok(()));
    let mut third = bind("14", BindingAct::Revoke, ARCH, "third@customer.example");
    third.closes = Some(architect_key().id);
    third.under = Some(genesis_id());
    assert!(judge(prior, &third).is_err());
}

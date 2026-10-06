//! A close ends the key in every namespace, whichever binding it names.

use crate::authority::fixture;
use crate::authority::{BindingAct, KeyBinding};
use crate::hash::VersionHash;
use crate::testkit;

use super::*;

const OWNER: &str = fixture::GENESIS_HOLDER;
const OTHER: &str = "architect@customer.example";

fn bind(tail: &str, act: BindingAct, principal: &str, ns: &str, key: Option<&str>, closes: Option<&KeyBinding>) -> KeyBinding {
    KeyBinding {
        id: format!("key:{}", fixture::ulid(tail)).parse().expect("id"),
        act,
        principal: testkit::identity(principal),
        namespace: ns.into(),
        key_type: key.map(|_| "ssh-ed25519".into()),
        key: key.map(Into::into),
        closes: closes.map(|c| c.id.clone()),
        self_bound: false,
        mandate: None,
        by: testkit::identity(principal),
        under: None,
        at: testkit::stamp("2026-10-06T00:00:00Z"),
        hash: VersionHash::zero(),
    }
}

#[test]
fn a_close_in_one_namespace_ends_the_same_key_in_every_other() {
    let in_a = bind("1", BindingAct::Add, OWNER, "a.ledger", Some("AAAAk"), None);
    let in_b = bind("2", BindingAct::Add, OWNER, "b.ledger", Some("AAAAk"), None);
    let revoke = bind("3", BindingAct::Revoke, OWNER, "a.ledger", None, Some(&in_a));
    let all = [&in_a, &in_b, &revoke];
    assert_eq!(closes_of(&all, &in_b).iter().map(|c| c.id.clone()).collect::<Vec<_>>(), vec![revoke.id.clone()]);
    assert!(is_closed(&all, &in_a) && is_closed(&all, &in_b));
}

#[test]
fn another_key_or_another_principals_binding_of_the_blob_stays_open() {
    let in_a = bind("1", BindingAct::Add, OWNER, "a.ledger", Some("AAAAk"), None);
    let other_key = bind("2", BindingAct::Add, OWNER, "a.ledger", Some("AAAAj"), None);
    let other_principal = bind("3", BindingAct::Add, OTHER, "a.ledger", Some("AAAAk"), None);
    let revoke = bind("4", BindingAct::Revoke, OWNER, "a.ledger", None, Some(&in_a));
    let all = [&in_a, &other_key, &other_principal, &revoke];
    assert!(!is_closed(&all, &other_key));
    assert!(!is_closed(&all, &other_principal));
}

#[test]
fn a_key_open_in_a_namespace_is_not_bound_there_twice_but_may_be_bound_elsewhere() {
    let first = bind("1", BindingAct::Add, OWNER, "a.ledger", Some("AAAAk"), None);
    let twice = bind("2", BindingAct::Add, OWNER, "a.ledger", Some("AAAAk"), None);
    let elsewhere = bind("3", BindingAct::Add, OWNER, "b.ledger", Some("AAAAk"), None);
    assert_eq!(already_open(&[&first], &twice).map(|o| o.id.clone()), Some(first.id.clone()));
    assert!(already_open(&[&first], &elsewhere).is_none());
    let revoke = bind("4", BindingAct::Revoke, OWNER, "a.ledger", None, Some(&first));
    assert!(already_open(&[&first, &revoke], &twice).is_none(), "a closed key is not open");
}

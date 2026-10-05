//! An earlier `at` never improves an act's verdict once the act has landed
//! after the entry it is judged against (#97, D6).
//!
//! Two grids, each enumerated whole with `std` (no property-testing
//! dependency). The act is an acceptance, or a `rev:` revocation of an
//! earlier acceptance; it lands after every entry, and every `at` is tried.
//!
//! - **Signed** (`dsse`): the policy and the act's key land first, and an
//!   optional key close (a rotate) and an optional revocation of the act's
//!   grant move over landing index and `at`. Key closes bite through `L011`
//!   and the review item; the grant revocation through `A006`.
//! - **Unsigned** (`[none]`): the namespace's first policy moves, with an
//!   optional grant revocation, and the act either names its grant or names
//!   none. A governed act naming no grant is `A006`; a pre-policy act is not
//!   checked (D5 (c)) — so a policy bites by governing the act at all. The
//!   holder's own further key `add` is on this grid too, after an optional
//!   key `revoke`: under `[none]` D7 is the only check that refuses one
//!   dated back inside the closed window.
//!
//! **Where the property does not hold, by ruling.** An act that landed no
//! later than an entry — in the same commit, or before it — is ordered
//! against it by `at` (D6: "in one commit, `at` decides"; a close dated to
//! the time of compromise reaches back over what landed in between). There
//! an earlier `at` legitimately helps, and `at_decides_only_where_the_act_
//! landed_no_later` pins that. Unavailability windows are the other stated
//! exception and are not on the grid.

use std::collections::BTreeSet;

#[path = "order_grid.rs"]
mod grid;

use grid::*;

fn placements() -> Vec<Option<Placed>> {
    let mut out = vec![None];
    for index in 0..2 {
        for at in TIMES {
            out.push(Some(Placed { index, at }));
        }
    }
    out
}

fn cases() -> Vec<Case> {
    let mut out = Vec::new();
    for act in [ActKind::Accept, ActKind::Revoke] {
        for grant_revoked in placements() {
            for close in placements() {
                out.push(Case { mode: Mode::Signed { close }, grant_revoked, act });
            }
            for policy in placements().into_iter().flatten() {
                for names_grant in [true, false] {
                    out.push(Case { mode: Mode::Unsigned { policy, names_grant, close: None }, grant_revoked, act });
                }
            }
        }
    }
    // The holder's own further key, after an optional key revoke.
    for policy in placements().into_iter().flatten() {
        for close in placements() {
            out.push(Case { mode: Mode::Unsigned { policy, names_grant: true, close }, grant_revoked: None, act: ActKind::Bind });
        }
    }
    out
}

/// The landing index of the last entry the act is judged against.
fn last_entry(case: Case) -> usize {
    let policy = match case.mode {
        Mode::Signed { close } => close.map_or(FIRST.index, |c| c.index),
        Mode::Unsigned { policy, close, .. } => close.map_or(policy.index, |c| c.index.max(policy.index)),
    };
    case.grant_revoked.map_or(policy, |r| r.index.max(policy))
}

#[test]
fn an_earlier_at_never_improves_an_act_landed_after_every_entry() {
    let mut checked = 0;
    let mut bit = 0;
    for case in cases() {
        let act_index = last_entry(case) + 1;
        let verdicts: Vec<_> = TIMES
            .iter()
            .map(|at| {
                let (store, landing, act) = build(case, act_index, at);
                verdict(&store, &landing, &act)
            })
            .collect();
        for (later, later_verdict) in verdicts.iter().enumerate() {
            for (earlier, earlier_verdict) in verdicts.iter().enumerate().take(later) {
                assert!(
                    later_verdict.is_subset(earlier_verdict),
                    "moving the act's `at` from {} back to {} improved its verdict from {later_verdict:?} to {earlier_verdict:?}: {case:?}, act landed at {act_index}",
                    TIMES[later],
                    TIMES[earlier]
                );
            }
        }
        checked += 1;
        bit += usize::from(verdicts.iter().any(|v| !v.is_empty()));
    }
    assert_eq!(checked, cases().len(), "the whole grid was visited");
    assert!(checked > 300, "{checked} cases");
    eprintln!("{bit} of {checked} cases carry a verdict an earlier `at` could have improved");
    assert!(bit * 3 > checked, "most of the grid carries a verdict to improve: {bit} of {checked}");
}

/// The grid is not vacuous: each entry, landed before the act, changes the
/// act's verdict on its own.
#[test]
fn each_entry_on_the_grid_bites() {
    let early = Placed { index: 0, at: TIMES[0] };
    let signed = Case { mode: Mode::Signed { close: None }, grant_revoked: None, act: ActKind::Accept };
    let at = |c: Case, index: usize, t: &str| {
        let (s, l, a) = build(c, index, t);
        verdict(&s, &l, &a)
    };
    assert!(at(signed, 1, TIMES[1]).is_empty(), "the bare signed point is clean: {:?}", at(signed, 1, TIMES[1]));
    assert!(at(Case { mode: Mode::Signed { close: Some(early) }, ..signed }, 1, TIMES[1]).contains("L011"), "a key close landed before the act");
    assert!(at(Case { grant_revoked: Some(early), ..signed }, 1, TIMES[1]).contains("A006"), "a grant revocation landed before the act");
    // A first policy landed before the act governs it although dated after
    // it: an act naming no grant is then `A006`.
    let late = Placed { index: 0, at: TIMES[2] };
    let unsigned = Case { mode: Mode::Unsigned { policy: late, names_grant: false, close: None }, grant_revoked: None, act: ActKind::Accept };
    assert!(at(unsigned, 1, TIMES[0]).contains("A006"), "a policy landed earlier governs an act dated before it");
    // Under `[none]`, a key revoke landed before the holder's own `add`
    // leaves them no live key: D7 refuses the add, however it is dated.
    let bind = Case { mode: Mode::Unsigned { policy: early, names_grant: true, close: None }, grant_revoked: None, act: ActKind::Bind };
    assert!(at(bind, 1, TIMES[0]).is_empty(), "an own add while the first key is open: {:?}", at(bind, 1, TIMES[0]));
    let closed = Case { mode: Mode::Unsigned { policy: early, names_grant: true, close: Some(Placed { index: 0, at: TIMES[2] }) }, ..bind };
    assert!(!at(closed, 1, TIMES[0]).is_empty(), "an own add dated back inside a window a landed revoke closed");
}

/// D6's own exception: an act that landed no later than an entry is
/// ordered against it by `at`, so there an earlier `at` does help.
#[test]
fn at_decides_only_where_the_act_landed_no_later() {
    let close = Placed { index: 1, at: TIMES[1] };
    let signed = Case { mode: Mode::Signed { close: Some(close) }, grant_revoked: None, act: ActKind::Accept };
    let at = |c: Case, index: usize, t: &str| {
        let (s, l, a) = build(c, index, t);
        verdict(&s, &l, &a)
    };
    assert!(!at(signed, 1, TIMES[0]).contains("L011"), "same commit as the close, dated before it: before it");
    assert!(at(signed, 1, TIMES[2]).contains("L011"), "same commit, dated after it: the close applies");
    let policy = Placed { index: 1, at: TIMES[1] };
    let unsigned = Case { mode: Mode::Unsigned { policy, names_grant: false, close: None }, grant_revoked: None, act: ActKind::Accept };
    assert!(at(unsigned, 1, TIMES[0]).is_empty(), "same commit as the first policy, dated before it: a pre-policy act");
    assert!(at(unsigned, 1, TIMES[2]).contains("A006"), "same commit, dated after it: governed");
}

#[path = "order_regressions_tests.rs"]
mod regressions;

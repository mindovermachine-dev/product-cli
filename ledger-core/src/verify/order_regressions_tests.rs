//! The holes #89's review rounds found, each as a named case on the grid.
//! The first — an entity appended to a landed file landing with its file —
//! is in landing itself, which a fixed landing cannot reach; it is pinned
//! by `ledger-cli/tests/immutability.rs`
//! `an_acceptance_appended_to_a_landed_file_after_the_close_fails_l011`.

use super::*;

fn at(c: Case, index: usize, t: &str) -> BTreeSet<String> {
    let (s, l, a) = build(c, index, t);
    verdict(&s, &l, &a)
}

/// Hole 2: a policy dated after a backdated act. The act landed after
/// the policy, so it is under it, however it is dated.
#[test]
fn a_policy_dated_after_a_backdated_act_still_governs_it() {
    let policy = Placed { index: 0, at: TIMES[2] };
    let case = Case { mode: Mode::Unsigned { policy, names_grant: false, close: None }, grant_revoked: None, act: ActKind::Accept };
    assert!(at(case, 1, TIMES[0]).contains("A006"), "{:?}", at(case, 1, TIMES[0]));
    let revoke = Case { act: ActKind::Revoke, ..case };
    assert!(at(revoke, 1, TIMES[0]).contains("A006"), "{:?}", at(revoke, 1, TIMES[0]));
}

/// Hole 3: a key close treated as enabling. Under `[none]` a holder's own
/// `add`, landed after the revoke that left them no key and dated back
/// inside the closed window, is refused by D7.
#[test]
fn a_key_close_is_terminating_not_enabling() {
    let policy = Placed { index: 0, at: TIMES[0] };
    let close = Placed { index: 0, at: TIMES[2] };
    let case = Case { mode: Mode::Unsigned { policy, names_grant: true, close: Some(close) }, grant_revoked: None, act: ActKind::Bind };
    assert!(at(case, 1, TIMES[1]).contains("SCHEMA"), "{:?}", at(case, 1, TIMES[1]));
    // Signed: an acceptance by the closed key, landed after the rotate
    // and dated before it, is `L011` — not a review item.
    let signed = Case { mode: Mode::Signed { close: Some(close) }, grant_revoked: None, act: ActKind::Accept };
    assert!(at(signed, 1, TIMES[1]).contains("L011"), "{:?}", at(signed, 1, TIMES[1]));
}

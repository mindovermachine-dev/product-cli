//! The order grid's machinery, one store per case.
//!
//! Each case builds a store, a fixed landing for it, and the act's verdict;
//! the property and its cases are in `order_tests.rs`.

use ed25519_dalek::{Signer, SigningKey};

use crate::authority::fixture::{self, accepted, genesis, grant, role};
use crate::authority::payload::{acceptance_bytes, binding_bytes, binding_hash, policy_hash, revocation_bytes, revocation_hash};
use crate::authority::{BindingAct, Capability, KeyBinding, Policy, Revocable, Revocation, Scheme};
use std::collections::BTreeSet;

use crate::hash::VersionHash;
use crate::landed::{key, revocation_key};
use crate::landing::Landing;
use crate::signing::dsse::{base64_encode, ed25519_blob, pae, PAYLOAD_TYPE};
use crate::signing::Sidecar;
use crate::store::Store;
use crate::testkit;

pub(super) const NS: &str = "fixture.ledger";
/// The fixture change-set's repo-relative path, under its namespace's directory.
pub(super) fn path() -> String {
    format!(".decisions/ns/{}/log/{}.yml", testkit::NS, testkit::CS_ULID)
}
/// The `at`s on the grid, for the entries and for the act alike.
pub(super) const TIMES: [&str; 3] = ["2026-10-02T09:00:00Z", "2026-10-02T10:00:00Z", "2026-10-02T11:00:00Z"];
pub(super) const HOLDER: &str = fixture::GENESIS_HOLDER;

/// One terminating or governing entry: where it landed and its `at`.
#[derive(Clone, Copy, Debug)]
pub(super) struct Placed {
    pub(super) index: usize,
    pub(super) at: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum ActKind {
    Accept,
    Revoke,
    /// The holder's own further key (`add`): D7 lets it through only while
    /// a key of theirs is open in the namespace.
    Bind,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Mode {
    /// `dsse` required; the policy and the key land first.
    Signed { close: Option<Placed> },
    /// `[none]`; the policy moves, and the act names its grant or none. A
    /// key close here is a `revoke`, which leaves the holder no live key.
    Unsigned { policy: Placed, names_grant: bool, close: Option<Placed> },
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Case {
    pub(super) mode: Mode,
    pub(super) grant_revoked: Option<Placed>,
    pub(super) act: ActKind,
}

/// The policy and the first key of the signed grid: landed first, dated
/// before everything else.
pub(super) const FIRST: Placed = Placed { index: 0, at: "2026-10-01T09:00:00Z" };

pub(super) fn k1() -> SigningKey {
    SigningKey::from_bytes(&[7u8; 32])
}

pub(super) fn k2() -> SigningKey {
    SigningKey::from_bytes(&[8u8; 32])
}

pub(super) fn sidecar(ulid: &str, key: &SigningKey, bytes: &[u8]) -> Sidecar {
    let sig = key.sign(&pae(PAYLOAD_TYPE, bytes));
    let envelope = serde_json::json!({
        "payloadType": PAYLOAD_TYPE,
        "payload": base64_encode(bytes),
        "signatures": [{"keyid": "", "sig": base64_encode(&sig.to_bytes())}],
    });
    Sidecar { namespace: NS.into(), ulid: ulid.into(), scheme: Scheme::Dsse, file: Sidecar::file_name(ulid, Scheme::Dsse), bytes: envelope.to_string().into_bytes() }
}

pub(super) fn binding(tail: &str, act: BindingAct, key: Option<&SigningKey>, closes: Option<&KeyBinding>, mandate: Option<String>, at: &str) -> KeyBinding {
    let holder = testkit::identity(HOLDER);
    let mut b = KeyBinding {
        id: format!("key:{}", fixture::ulid(tail)).parse().expect("id"),
        act,
        principal: holder.clone(),
        namespace: NS.into(),
        key_type: key.map(|_| "ssh-ed25519".into()),
        key: key.map(|k| ed25519_blob(&k.verifying_key())),
        closes: closes.map(|c| c.id.clone()),
        self_bound: mandate.is_some(),
        mandate,
        by: holder,
        under: None,
        at: testkit::stamp(at),
        hash: VersionHash::zero(),
    };
    b.hash = binding_hash(&b);
    b
}

pub(super) fn acceptance(tail: &str, version: &crate::version::VersionRaw, under: Option<&crate::authority::Grant>, at: &str) -> crate::acceptance::Acceptance {
    let mut a = testkit::acceptance(version);
    a.id = format!("acc:{}", fixture::ulid(tail)).parse().expect("acc");
    a.actor = testkit::identity(HOLDER);
    a.under = under.map(|g| g.id.clone());
    a.at = testkit::stamp(at);
    a
}

pub(super) fn revocation(tail: &str, revokes: Revocable, under: &crate::authority::Grant, at: &str) -> Revocation {
    let mut r = Revocation {
        id: Some(format!("rev:{}", fixture::ulid(tail)).parse().expect("rev")),
        revokes: Some(revokes),
        acceptance: None,
        at: testkit::stamp(at),
        actor: Some(testkit::identity(HOLDER)),
        by: None,
        reason: "order grid".into(),
        under: Some(under.id.clone()),
        hash: None,
    };
    r.hash = Some(revocation_hash(&r));
    r
}

/// A store under construction: its change-set, its sidecars, and where
/// each entity lands.
pub(super) struct Builder {
    pub(super) cs: crate::changeset::ChangeSet,
    pub(super) sidecars: Vec<Sidecar>,
    pub(super) placed: Vec<(String, usize)>,
    pub(super) signed: bool,
}

impl Builder {
    fn sign(&mut self, ulid: &str, bytes: &[u8]) {
        if self.signed {
            self.sidecars.push(sidecar(ulid, &k1(), bytes));
        }
    }

    /// The first key, self-bound (in the signed grid after the policy: a
    /// binding before a namespace's first policy is never trusted), and
    /// its close: a rotate signed by the key it closes, or under `[none]` a
    /// revoke that leaves the holder no live key.
    pub(super) fn keys(&mut self, mode: Mode, g: &crate::authority::Grant) {
        let first = binding("4", BindingAct::Add, Some(&k1()), None, g.external_ref.clone(), "2026-10-01T09:10:00Z");
        self.sidecars.push(sidecar(first.id.ulid(), &k1(), &binding_bytes(&first)));
        self.placed.push((key("key_bindings", &first.id.to_string()), 0));
        let close = match mode {
            Mode::Signed { close } => close.map(|c| (binding("7", BindingAct::Rotate, Some(&k2()), Some(&first), None, c.at), c)),
            Mode::Unsigned { close, .. } => close.map(|c| (binding("7", BindingAct::Revoke, None, Some(&first), None, c.at), c)),
        };
        self.cs.key_bindings.push(first);
        if let Some((closing, c)) = close {
            self.sidecars.push(sidecar(closing.id.ulid(), &k1(), &binding_bytes(&closing)));
            self.placed.push((key("key_bindings", &closing.id.to_string()), c.index));
            self.cs.key_bindings.push(closing);
        }
    }

    /// The act, dated `at` and landed at `index`; returns its id.
    pub(super) fn act(&mut self, kind: ActKind, version: &crate::version::VersionRaw, ga: &crate::authority::Grant, under: Option<&crate::authority::Grant>, at: &str, index: usize) -> String {
        match kind {
            ActKind::Accept => {
                let a = acceptance("9", version, under, at);
                self.sign(a.id.ulid(), &acceptance_bytes(&a));
                self.placed.push((key("acceptances", &a.id.to_string()), index));
                let id = a.id.to_string();
                self.cs.acceptances.push(a);
                id
            }
            ActKind::Bind => {
                let b = binding("12", BindingAct::Add, Some(&k2()), None, None, at);
                self.sign(b.id.ulid(), &binding_bytes(&b));
                self.placed.push((key("key_bindings", &b.id.to_string()), index));
                let id = b.id.to_string();
                self.cs.key_bindings.push(b);
                id
            }
            ActKind::Revoke => self.revoke_earlier(version, ga, under, at, index),
        }
    }

    /// An earlier acceptance, landed with the enabling entries and dated
    /// before every entry on the grid, then its revocation (the act).
    fn revoke_earlier(&mut self, version: &crate::version::VersionRaw, ga: &crate::authority::Grant, under: Option<&crate::authority::Grant>, at: &str, index: usize) -> String {
        let earlier = acceptance("10", version, Some(ga), "2026-10-01T10:00:00Z");
        self.sign(earlier.id.ulid(), &acceptance_bytes(&earlier));
        self.placed.push((key("acceptances", &earlier.id.to_string()), 0));
        let r = revocation("11", Revocable::Acceptance(earlier.id.clone()), ga, at);
        let r = Revocation { under: under.map(|g| g.id.clone()), ..r };
        let r = Revocation { hash: Some(revocation_hash(&r)), ..r };
        let ulid = r.id.as_ref().map(|i| i.ulid().to_string()).unwrap_or_default();
        self.sign(&ulid, &revocation_bytes(&r));
        self.placed.push((revocation_key(&r), index));
        self.cs.acceptances.push(earlier);
        let id = r.subject();
        self.cs.revocations.push(r);
        id
    }
}

/// The namespace's policy for `case`: `dsse` landed first in the signed
/// grid, `[none]` where the grid puts it otherwise.
pub(super) fn policy(case: Case, g: &crate::authority::Grant) -> (Policy, usize) {
    let (placed, schemes) = match case.mode {
        Mode::Signed { .. } => (FIRST, vec![Scheme::Dsse]),
        Mode::Unsigned { policy, .. } => (policy, vec![Scheme::None]),
    };
    let mut p = Policy {
        id: format!("pol:{}", fixture::ulid("3")).parse().expect("id"),
        namespace: NS.into(),
        schemes,
        require_sk: false,
        accept_role: "acceptor".into(),
        reaccept_within_days: None,
        replaces: None,
        by: testkit::identity(HOLDER),
        under: Some(g.id.clone()),
        at: testkit::stamp(placed.at),
        hash: VersionHash::zero(),
    };
    p.hash = policy_hash(&p);
    (p, placed.index)
}

/// The store for `case` with the act dated `act_at` and landed at
/// `act_index`, its landing, and the act's id.
pub(super) fn build(case: Case, act_index: usize, act_at: &str) -> (Store, Landing, String) {
    let g = genesis("1", "steward");
    let ga = grant("5", "acceptor", HOLDER, &format!("ns:{NS}"), 0);
    let mut version = testkit::version();
    version.decision = format!("dec:{NS}/01K2C4YQJ3F8M0PT5W7NZ9RDXA").parse().expect("dec");
    let version = testkit::sealed(version);
    let (policy, policy_index) = policy(case, &g);
    let names_grant = match case.mode {
        Mode::Signed { .. } => true,
        Mode::Unsigned { names_grant, .. } => names_grant,
    };
    let mut cs = fixture::changeset(vec![g.clone(), ga.clone()], vec![accepted("2", &g), accepted("6", &ga)]);
    cs.versions = vec![version.clone()];
    let placed = vec![
        (key("grants", &g.id.to_string()), 0),
        (key("grants", &ga.id.to_string()), 0),
        (key("grant_acceptances", &cs.grant_acceptances[0].id.to_string()), 0),
        (key("grant_acceptances", &cs.grant_acceptances[1].id.to_string()), 0),
        (key("policies", &policy.id.to_string()), policy_index),
    ];
    cs.policies = vec![policy];
    let mut b = Builder { cs, sidecars: Vec::new(), placed, signed: matches!(case.mode, Mode::Signed { .. }) };
    b.keys(case.mode, &g);
    if let Some(r) = case.grant_revoked {
        let rev = revocation("8", Revocable::Grant(ga.id.clone()), &g, r.at);
        b.placed.push((revocation_key(&rev), r.index));
        b.cs.revocations.push(rev);
    }
    let act_id = b.act(case.act, &version, &ga, names_grant.then_some(&ga), act_at, act_index);
    let mut store = fixture::store(vec![role("steward", Capability::ROOT), role("acceptor", &[Capability::AcceptDecision])], b.cs);
    store.sidecars = b.sidecars;
    // The role files land with the enabling entries: a role takes effect
    // from its file's landing.
    let roles: Vec<(String, usize)> = store
        .roles
        .iter()
        .map(|r| (format!("{}/{}", crate::layout::relative_dir(&r.namespace, crate::layout::Kind::Role), r.file_name()), 0))
        .collect();
    (store, Landing::fixed(&path(), &b.placed, &roles, 10), act_id)
}

/// What the gate says about the act alone: each file-gate and graph-stage
/// class against it, and whether it is a review item.
pub(super) fn verdict(store: &Store, landing: &Landing, act: &str) -> BTreeSet<String> {
    let today = testkit::date("2026-10-15");
    let signing = crate::signing::check::check(store, landing, today);
    let mut out: BTreeSet<String> = signing.findings.iter().filter(|f| f.subject == act).map(|f| f.class.code().to_string()).collect();
    out.extend(crate::verify::acts::unauthorised(store, landing).iter().filter(|f| f.subject == act).map(|f| f.class.code().to_string()));
    if signing.reaccept.iter().any(|r| r.acceptance == act) {
        out.insert("review".into());
    }
    out
}


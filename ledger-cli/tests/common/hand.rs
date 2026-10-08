//! Writing entities by hand — what `verify` must judge without the verbs.
//!
//! The verbs refuse what the gate would fail; these helpers write the file
//! a person (or a forger) could write anyway: an acceptance with any `at`,
//! any `under`, signed by any key or by none, straight into the log.

use chrono::{DateTime, Utc};
use ledger_core::acceptance::{Acceptance, AcceptanceScope};
use ledger_core::changeset::ChangeSet;
use ledger_core::mint::UlidMint;

use super::Repo;

/// Sign `bytes` in `ns` with the private key at `key`, as `ssh-keygen` does.
pub fn ssh_sign(key: &str, ns: &str, bytes: &[u8]) -> Vec<u8> {
    ledger_core::signing::ssh::sign(std::path::Path::new(key), ns, bytes).expect("ssh-keygen -Y sign")
}

/// The latest version hash of `decision`.
pub fn tip(repo: &Repo, decision: &str) -> String {
    let store = ledger_core::store::load(repo.path());
    let view = ledger_core::verify::view::View::build(&store);
    view.latest
        .get(decision)
        .and_then(|i| view.versions.get(*i))
        .map(|v| v.raw.hash.to_string())
        .expect("a filed version")
}

/// What a hand-filed acceptance says.
pub struct HandAccept<'a> {
    pub decision: &'a str,
    pub actor: &'a str,
    pub at: DateTime<Utc>,
    pub under: Option<&'a str>,
    /// The private key to sign with, or none.
    pub key: Option<&'a str>,
}

/// File an acceptance by hand; returns its id.
pub fn accept(repo: &Repo, h: &HandAccept<'_>) -> String {
    let mut mint = UlidMint::system();
    let acceptance = Acceptance {
        id: mint.mint_id("acc").expect("acc id"),
        decision: h.decision.parse().expect("decision"),
        version: tip(repo, h.decision).parse().expect("hash"),
        actor: h.actor.parse().expect("actor"),
        at: h.at,
        scope: AcceptanceScope::Version,
        expires_at: None,
        under: h.under.map(|g| g.parse().expect("grant id")),
        signature: String::new(),
    };
    let ns = acceptance.decision.namespace().to_string();
    let bytes = ledger_core::authority::payload::acceptance_bytes(&acceptance);
    let id = acceptance.id.to_string();
    let ulid = acceptance.id.ulid().to_string();
    let mut cs = ChangeSet::empty(7, mint.mint_id("cs").expect("cs id"), h.at, h.actor.parse().expect("actor"), None);
    cs.acceptances.push(acceptance);
    let text = serde_yaml::to_string(&cs).expect("yaml");
    std::fs::write(repo.path().join(".decisions/log").join(cs.file_name()), text).expect("write log");
    if let Some(key) = h.key {
        let sig = ssh_sign(key, &ns, &bytes);
        let dir = repo.path().join(".decisions/sig");
        std::fs::create_dir_all(&dir).expect("sig dir");
        std::fs::write(dir.join(format!("{ulid}.ssh.sig")), sig).expect("write sig");
    }
    id
}

/// Commit everything, as the current git identity.
pub fn commit(repo: &Repo, message: &str) {
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-q", "-m", message]);
}

/// The `at` of the newest key binding in the store, to date hand acts by.
pub fn last_binding_at(repo: &Repo) -> DateTime<Utc> {
    let store = ledger_core::store::load(repo.path());
    store.log.iter().flat_map(|l| l.file.key_bindings.iter()).map(|b| b.at).max().expect("a binding")
}

/// The first word of `out` starting with `prefix`, trimmed of punctuation.
pub fn word(out: &str, prefix: &str) -> String {
    out.split_whitespace()
        .map(|w| w.trim_matches(|c: char| c == '(' || c == ')' || c == '`' || c == ','))
        .find(|w| w.starts_with(prefix))
        .unwrap_or_else(|| panic!("no {prefix} in {out}"))
        .to_string()
}

/// File a change-set holding exactly `bindings` and `policies`, each signed
/// with the matching key in `signers` (by entity ULID) — or unsigned.
pub fn file(
    repo: &Repo,
    bindings: Vec<ledger_core::authority::KeyBinding>,
    policies: Vec<ledger_core::authority::Policy>,
    signers: &[(&str, &str)],
) {
    let mut mint = UlidMint::system();
    let at = bindings.iter().map(|b| b.at).chain(policies.iter().map(|p| p.at)).max().unwrap_or_else(Utc::now);
    let who = bindings.first().map(|b| b.by.clone()).or_else(|| policies.first().map(|p| p.by.clone())).expect("an entity");
    let mut cs = ChangeSet::empty(7, mint.mint_id("cs").expect("cs id"), at, who, None);
    let dir = repo.path().join(".decisions/sig");
    std::fs::create_dir_all(&dir).expect("sig dir");
    for b in &bindings {
        let ulid = b.id.ulid().to_string();
        if let Some((_, key)) = signers.iter().find(|(u, _)| *u == ulid) {
            let sig = ssh_sign(key, &b.namespace, &ledger_core::authority::payload::binding_bytes(b));
            std::fs::write(dir.join(format!("{ulid}.ssh.sig")), sig).expect("sig");
        }
    }
    for p in &policies {
        let ulid = p.id.ulid().to_string();
        if let Some((_, key)) = signers.iter().find(|(u, _)| *u == ulid) {
            let sig = ssh_sign(key, &p.namespace, &ledger_core::authority::payload::policy_bytes(p));
            std::fs::write(dir.join(format!("{ulid}.ssh.sig")), sig).expect("sig");
        }
    }
    cs.key_bindings = bindings;
    cs.policies = policies;
    let text = serde_yaml::to_string(&cs).expect("yaml");
    std::fs::write(repo.path().join(".decisions/log").join(cs.file_name()), text).expect("write log");
}

/// A sealed key binding, as a hand would write it.
pub fn binding(
    act: ledger_core::authority::BindingAct,
    principal: &str,
    by: &str,
    ns: &str,
    key_pub: Option<&str>,
    closes: Option<&str>,
    under: Option<&str>,
) -> ledger_core::authority::KeyBinding {
    let (key_type, key) = match key_pub {
        Some(path) => {
            let text = std::fs::read_to_string(path).expect("pub key");
            let mut parts = text.split_whitespace();
            (parts.next().map(str::to_string), parts.next().map(str::to_string))
        }
        None => (None, None),
    };
    let mut b = ledger_core::authority::KeyBinding {
        id: UlidMint::system().mint_id("key").expect("key id"),
        act,
        principal: principal.parse().expect("principal"),
        namespace: ns.to_string(),
        key_type,
        key,
        closes: closes.map(|c| c.parse().expect("binding id")),
        self_bound: false,
        mandate: None,
        by: by.parse().expect("by"),
        under: under.map(|g| g.parse().expect("grant id")),
        at: Utc::now(),
        hash: ledger_core::hash::VersionHash::zero(),
    };
    b.hash = ledger_core::authority::payload::binding_hash(&b);
    b
}

/// The log file holding the entity `id`, by its text.
pub fn file_holding(repo: &Repo, id: &str) -> std::path::PathBuf {
    std::fs::read_dir(repo.path().join(".decisions/log"))
        .expect("log")
        .flatten()
        .map(|e| e.path())
        .find(|p| std::fs::read_to_string(p).is_ok_and(|t| t.contains(&format!("id: {id}"))))
        .unwrap_or_else(|| panic!("no log file holds {id}"))
}

/// Move the acceptance `id` out of its own change-set and append it to the
/// landed file `into` — as a hand editing a landed file would. The target's
/// `format:` is raised to 7 when lower: that changes no entity, but a raise
/// in the same step as an append is no correction (LP-3.16, ruling 58), so
/// `verify` reports it as `L007` on the declaration.
pub fn append_into(repo: &Repo, id: &str, into: &std::path::Path) {
    use serde_yaml::Value;
    let from = file_holding(repo, id);
    let source: Value = serde_yaml::from_str(&std::fs::read_to_string(&from).expect("read")).expect("yaml");
    let item = source.get("acceptances").and_then(Value::as_sequence).and_then(|s| s.first()).cloned().expect("acceptance");
    let mut target: Value = serde_yaml::from_str(&std::fs::read_to_string(into).expect("read")).expect("yaml");
    let map = target.as_mapping_mut().expect("mapping");
    if map.get("format").and_then(Value::as_u64).unwrap_or(0) < 7 {
        map.insert("format".into(), 7.into());
    }
    let list = map.entry("acceptances".into()).or_insert_with(|| Value::Sequence(Vec::new()));
    list.as_sequence_mut().expect("list").push(item);
    std::fs::write(into, serde_yaml::to_string(&target).expect("yaml")).expect("write");
    std::fs::remove_file(from).expect("remove");
}

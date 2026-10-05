//! What the inbox lists for one holder (registry PRD §5): every proposed
//! decision in a namespace where the holder may accept, per branch, with the
//! grant it would be accepted under.
//!
//! The proposed decisions and their fields are read from each branch's named
//! graph. Whether the holder may accept one — and under which grant — is the
//! role check `accept` itself runs (`authority::authorize`), over that
//! branch's store as committed, so the grant shown is the grant `accept
//! --batch` resolves; a second derivation in SPARQL would be a second copy
//! of the role check. A namespace with no policy has no holders: its
//! proposed decisions are in nobody's list, and it is named as unchecked.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use super::index::{Branch, Index};
use super::terms::{iri, literal};
use crate::authority::{authorize, Act, Authority, Denial, Target};
use crate::identity::Identity;

/// One proposed decision, as the holder sees it.
#[derive(Debug, Clone, Serialize)]
pub struct Item {
    pub repository: String,
    pub branch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr: Option<u32>,
    pub decision: String,
    pub version: String,
    pub statement: String,
    pub set: String,
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// The grant it would be accepted under (D9), or why none is chosen yet.
    pub grant: String,
    pub proposer: String,
    /// `agent` when the proposer is a model or bot identity, else `human`.
    pub identity_class: &'static str,
    /// The change-set's note: the rationale as filed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    /// `based_on` pointers whose scheme names code (`symbol:`, `code:`).
    pub citing: Vec<String>,
    /// A `based_on` pointer naming a rule (`rule:`, `analyzer:`), if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule: Option<String>,
    /// Every other `based_on` pointer, as filed.
    pub grounds: Vec<String>,
    /// Against the predecessor version, when this is a revision.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub diff: Vec<String>,
    /// When the proposing change-set was made — the sort key, newest first.
    pub when: String,
}

/// A namespace a branch speaks that has no policy: unchecked, and listed
/// rather than omitted.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Unchecked {
    pub repository: String,
    pub branch: String,
    pub namespace: String,
}

/// An acceptance of the holder's awaiting re-acceptance on one branch.
#[derive(Debug, Clone, Serialize)]
pub struct Review {
    pub repository: String,
    pub branch: String,
    pub decision: String,
    pub version: String,
    pub acceptance: String,
    /// `key-closed` (`L012`: affirm by re-accepting) or `fails-on-base`
    /// (`L011` against the base: re-accepting does not clear it).
    pub reason: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,
}

/// Everything the holder's list holds.
#[derive(Debug, Clone, Serialize)]
pub struct Listing {
    pub items: Vec<Item>,
    pub unchecked: Vec<Unchecked>,
}

const PROPOSED: &str = "SELECT ?dec ?id ?ns ?v ?hash ?statement ?set ?key ?proposer ?when ?note ?prevStatement WHERE { \
    ?v a <urn:ledger:ns#DecisionVersion> ; <urn:ledger:ns#ofDecision> ?dec ; <urn:ledger:ns#hash> ?hash ; \
       <urn:ledger:ns#statement> ?statement ; <urn:ledger:ns#set> ?set ; <http://www.w3.org/ns/prov#wasGeneratedBy> ?cs . \
    ?dec <urn:ledger:ns#namespace> ?ns ; <urn:ledger:ns#id> ?id . \
    FILTER NOT EXISTS { ?c <http://www.w3.org/ns/prov#wasRevisionOf> ?v } \
    FILTER NOT EXISTS { ?m <urn:ledger:ns#mergedFrom> ?v } \
    FILTER NOT EXISTS { ?s <urn:ledger:ns#supersedes> ?dec } \
    FILTER NOT EXISTS { ?a <urn:ledger:ns#signsVersion> ?v . FILTER NOT EXISTS { ?r <urn:ledger:ns#revokes> ?a } } \
    OPTIONAL { ?v <urn:ledger:ns#key> ?key } \
    OPTIONAL { ?cs <http://www.w3.org/ns/prov#wasAssociatedWith> ?proposer } \
    OPTIONAL { ?cs <http://www.w3.org/ns/prov#endedAtTime> ?when } \
    OPTIONAL { ?cs <urn:ledger:ns#note> ?note } \
    OPTIONAL { ?v <http://www.w3.org/ns/prov#wasRevisionOf> ?prev . ?prev <urn:ledger:ns#statement> ?prevStatement } }";

const BASED_ON: &str = "SELECT ?v ?b WHERE { ?v <urn:ledger:ns#basedOn> ?b }";

const NAMESPACES: &str = "SELECT DISTINCT ?ns WHERE { ?d a <urn:ledger:ns#Decision> ; <urn:ledger:ns#namespace> ?ns }";

/// The holder's list over the whole index. `as_role` is the batch's `--as`.
pub fn list(index: &Index, holder: &Identity, as_role: Option<&str>) -> Result<Listing, String> {
    let mut items = Vec::new();
    let mut unchecked = BTreeSet::new();
    for branch in &index.branches {
        let path = index.path(&branch.repository).ok_or("unknown repository")?;
        let store = crate::revision::load_at(&path, &branch.rev).map_err(|e| format!("{} `{}`: {e}", branch.repository, branch.name))?;
        let auth = Authority::build(&store);
        for row in index.dataset.select_in(&branch.graph, NAMESPACES)? {
            let ns = literal(row.get("ns").map(String::as_str).unwrap_or_default());
            if auth.policy(&ns).is_none() {
                unchecked.insert(Unchecked { repository: branch.repository.clone(), branch: branch.name.clone(), namespace: ns });
            }
        }
        let grounds = based_on(index, branch)?;
        for row in index.dataset.select_in(&branch.graph, PROPOSED)? {
            if let Some(item) = item(branch, &row, &grounds, &auth, holder, as_role) {
                items.push(item);
            }
        }
    }
    items.sort_by(|a, b| {
        (&a.repository, a.pr.is_none(), a.pr, &a.branch, &b.when, &a.decision)
            .cmp(&(&b.repository, b.pr.is_none(), b.pr, &b.branch, &a.when, &b.decision))
    });
    Ok(Listing { items, unchecked: unchecked.into_iter().collect() })
}

fn based_on(index: &Index, branch: &Branch) -> Result<BTreeMap<String, Vec<String>>, String> {
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for row in index.dataset.select_in(&branch.graph, BASED_ON)? {
        let (Some(v), Some(b)) = (row.get("v"), row.get("b")) else { continue };
        out.entry(v.clone()).or_default().push(literal(b));
    }
    for list in out.values_mut() {
        list.sort();
    }
    Ok(out)
}

fn item(
    branch: &Branch,
    row: &BTreeMap<String, String>,
    grounds: &BTreeMap<String, Vec<String>>,
    auth: &Authority<'_>,
    holder: &Identity,
    as_role: Option<&str>,
) -> Option<Item> {
    let get = |k: &str| row.get(k).map(|t| literal(t));
    let namespace = get("ns")?;
    let set = row.get("set").map(|t| iri(t).trim_start_matches("urn:ledger-set:").to_string())?;
    let grant = holder_grant(auth, holder, &namespace, &set, as_role)?;
    let proposer = row.get("proposer").map(|t| iri(t)).unwrap_or_default();
    let class = match proposer.parse::<Identity>().ok().and_then(|i| i.model_or_bot_reason()) {
        Some(_) => "agent",
        None => "human",
    };
    let pointers = row.get("v").and_then(|v| grounds.get(v)).cloned().unwrap_or_default();
    let (citing, rule, rest) = classify(pointers);
    let statement = get("statement")?;
    Some(Item {
        repository: branch.repository.clone(),
        branch: branch.name.clone(),
        pr: branch.pr,
        decision: get("id")?,
        version: get("hash")?,
        diff: get("prevStatement").map(|old| diff(&old, &statement)).unwrap_or_default(),
        statement,
        set,
        namespace,
        key: get("key"),
        grant,
        proposer,
        identity_class: class,
        rationale: get("note"),
        citing,
        rule,
        grounds: rest,
        when: get("when").unwrap_or_default(),
    })
}

/// The grant the holder would accept under, by the role check `accept`
/// runs; `None` when the namespace has no policy or the holder holds no
/// qualifying grant (the item is then not theirs to list).
fn holder_grant(auth: &Authority<'_>, holder: &Identity, ns: &str, set: &str, as_role: Option<&str>) -> Option<String> {
    let policy = auth.policy(ns)?;
    let target = Target::Decision { namespace: ns, set };
    match authorize(auth, holder, Act::Accept, target, chrono::Utc::now(), Some(&policy.accept_role), as_role) {
        Ok(a) => Some(a.grant),
        Err(Denial::Ambiguous(roles)) => Some(format!("needs --as ({})", roles.join(", "))),
        Err(_) => None,
    }
}

/// Split `based_on` pointers by scheme: code it cites, the rule that fired,
/// and every other ground as filed.
fn classify(pointers: Vec<String>) -> (Vec<String>, Option<String>, Vec<String>) {
    let scheme = |p: &str| p.split_once(':').map(|(s, _)| s.to_string()).unwrap_or_default();
    let mut citing = Vec::new();
    let mut rule = None;
    let mut rest = Vec::new();
    for p in pointers {
        match scheme(&p).as_str() {
            "symbol" | "code" => citing.push(p),
            "rule" | "analyzer" if rule.is_none() => rule = Some(p),
            _ => rest.push(p),
        }
    }
    (citing, rule, rest)
}

/// A line diff from the predecessor's statement to this one.
fn diff(old: &str, new: &str) -> Vec<String> {
    let mut out: Vec<String> = old.lines().map(|l| format!("- {l}")).collect();
    out.extend(new.lines().map(|l| format!("+ {l}")));
    out
}

/// The grant the holder would re-accept `decision` under on `branch` — the
/// same role check, for an affirmation row that is not a proposed item.
pub fn grant_on(index: &Index, branch: &Branch, holder: &Identity, decision: &str, as_role: Option<&str>) -> Option<String> {
    let path = index.path(&branch.repository)?;
    let store = crate::revision::load_at(&path, &branch.rev).ok()?;
    let view = crate::verify::view::View::build(&store);
    let set = view.latest.get(decision).and_then(|i| view.versions.get(*i)).map(|v| v.raw.set.clone())?;
    let ns = decision.strip_prefix("dec:")?.split('/').next()?.to_string();
    holder_grant(&Authority::build(&store), holder, &ns, &set, as_role)
}

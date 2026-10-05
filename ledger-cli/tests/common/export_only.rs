//! An export-only verifier (#70): its whole input is one namespace's
//! N-Triples export and the sidecar files. No file under `.decisions/`, no
//! git. It rebuilds each signed entity's bytes from the export's triples by
//! the format document's reconstruction rules, rebuilds `allowed_signers`
//! from the `KeyBinding` nodes, and verifies every signature with
//! `ssh-keygen` at the entity's own `at`. Its stated limit (D6): it orders
//! by `at` alone, so it cannot check landing order.

use std::collections::BTreeMap;
use std::path::Path;

use ledger_core::canon::{put, put_set};
use serde_json::{Map, Value};

const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const LEDGER: &str = "urn:ledger:ns#";
const PROV: &str = "http://www.w3.org/ns/prov#";

/// One object: an IRI or a literal's lexical form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term {
    Iri(String),
    Literal(String),
}

impl Term {
    fn text(&self) -> &str {
        match self {
            Self::Iri(s) | Self::Literal(s) => s,
        }
    }
}

/// Subject → predicate → objects.
pub type Graph = BTreeMap<String, BTreeMap<String, Vec<Term>>>;

/// Parse canonical N-Triples (the export's form).
pub fn parse(text: &str) -> Graph {
    let mut g = Graph::new();
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
        let (s, rest) = iri_at(line).expect("subject");
        let (p, rest) = iri_at(rest.trim_start()).expect("predicate");
        let rest = rest.trim_start();
        let object = if rest.starts_with('<') {
            Term::Iri(iri_at(rest).expect("object").0)
        } else {
            Term::Literal(literal_at(rest))
        };
        g.entry(s).or_default().entry(p).or_default().push(object);
    }
    g
}

fn iri_at(s: &str) -> Option<(String, &str)> {
    let body = s.strip_prefix('<')?;
    let end = body.find('>')?;
    Some((body[..end].to_string(), &body[end + 1..]))
}

fn literal_at(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.strip_prefix('"').expect("literal").chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => break,
            '\\' => match chars.next() {
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some('u') => {
                    let hex: String = chars.by_ref().take(4).collect();
                    out.push(char::from_u32(u32::from_str_radix(&hex, 16).unwrap_or(0xFFFD)).unwrap_or('\u{FFFD}'));
                }
                Some(other) => out.push(other),
                None => {}
            },
            c => out.push(c),
        }
    }
    out
}

fn one<'a>(g: &'a Graph, s: &str, p: &str) -> Option<&'a str> {
    g.get(s)?.get(p)?.first().map(Term::text)
}

fn all<'a>(g: &'a Graph, s: &str, p: &str) -> Vec<&'a str> {
    g.get(s).and_then(|m| m.get(p)).map(|v| v.iter().map(Term::text).collect()).unwrap_or_default()
}

fn is(g: &Graph, s: &str, class: &str) -> bool {
    all(g, s, RDF_TYPE).contains(&format!("{LEDGER}{class}").as_str())
}

fn strip(v: Option<&str>, prefix: &str) -> Option<String> {
    v.map(|x| x.strip_prefix(prefix).unwrap_or(x).to_string())
}

fn l(p: &str) -> String {
    format!("{LEDGER}{p}")
}

fn pv(p: &str) -> String {
    format!("{PROV}{p}")
}

fn bytes(prefix: &str, m: Map<String, Value>) -> Vec<u8> {
    ledger_core::hash::signed_bytes(prefix, Value::Object(m).to_string().as_bytes())
}

/// The signed bytes of the entity `node`, rebuilt from the export alone.
pub fn rebuild(g: &Graph, node: &str) -> Option<Vec<u8>> {
    let at = one(g, node, &pv("generatedAtTime")).map(str::to_string);
    let by = strip(one(g, node, &pv("wasAttributedTo")), "mailto:");
    let under = strip(one(g, node, &l("under")), "urn:");
    let mut m = Map::new();
    if is(g, node, "Acceptance") {
        put(&mut m, "decision", strip(one(g, node, &l("ofDecision")), "urn:"));
        put(&mut m, "version", strip(one(g, node, &l("signsVersion")), "urn:"));
        put(&mut m, "actor", by);
        put(&mut m, "at", at);
        put(&mut m, "scope", one(g, node, &l("scope")).map(str::to_string));
        put(&mut m, "expires_at", one(g, node, &l("expiresAt")).map(str::to_string));
        put(&mut m, "under", under);
        return Some(bytes("ledger.acceptance.v1", m));
    }
    if is(g, node, "Revocation") {
        put(&mut m, "revokes", strip(one(g, node, &l("revokes")), "urn:"));
        put(&mut m, "actor", by);
        put(&mut m, "at", at);
        put(&mut m, "reason", one(g, node, &l("revocationReason")).map(str::to_string));
        put(&mut m, "under", under);
        return Some(bytes("ledger.revocation.v1", m));
    }
    if is(g, node, "KeyBinding") {
        put(&mut m, "id", one(g, node, &l("id")).map(str::to_string));
        put(&mut m, "act", one(g, node, &l("bindingAct")).map(str::to_string));
        put(&mut m, "principal", strip(one(g, node, &l("principal")), "mailto:"));
        put(&mut m, "namespace", one(g, node, &l("namespace")).map(str::to_string));
        put(&mut m, "key_type", one(g, node, &l("keyType")).map(str::to_string));
        put(&mut m, "key", one(g, node, &l("publicKey")).map(str::to_string));
        put(&mut m, "closes", strip(one(g, node, &l("closes")), "urn:"));
        put(&mut m, "self_bound", one(g, node, &l("selfBound")).map(str::to_string));
        put(&mut m, "mandate", one(g, node, &l("mandate")).map(str::to_string));
        put(&mut m, "by", by);
        put(&mut m, "under", under);
        put(&mut m, "at", at);
        return Some(bytes("ledger.identity-binding.v1", m));
    }
    if is(g, node, "NamespacePolicy") {
        put(&mut m, "id", one(g, node, &l("id")).map(str::to_string));
        put(&mut m, "namespace", one(g, node, &l("namespace")).map(str::to_string));
        put_set(&mut m, "schemes", all(g, node, &l("requiresScheme")).into_iter().map(str::to_string));
        put(&mut m, "require_sk", one(g, node, &l("requiresSecurityKey")).map(str::to_string));
        put(&mut m, "accept_role", strip(one(g, node, &l("acceptRole")), "urn:ledger-role:"));
        put(&mut m, "reaccept_within_days", one(g, node, &l("reacceptWithinDays")).map(str::to_string));
        put(&mut m, "replaces", one(g, node, &l("replacesPolicy")).map(str::to_string));
        put(&mut m, "by", by);
        put(&mut m, "under", under);
        // A policy that carries a signature is a format-7 change: `at` is in
        // its payload (D8).
        put(&mut m, "at", at);
        return Some(bytes("ledger.namespace-policy.v1", m));
    }
    None
}

fn keygen_time(lexical: &str) -> String {
    lexical.chars().filter(char::is_ascii_digit).collect::<String>() + "Z"
}

/// `allowed_signers` from the export's `KeyBinding` nodes.
pub fn allowed_signers(g: &Graph) -> String {
    let bindings: Vec<&String> = g.keys().filter(|s| is(g, s, "KeyBinding")).collect();
    let mut lines = Vec::new();
    for b in &bindings {
        let act = one(g, b, &l("bindingAct")).unwrap_or_default();
        if act != "add" && act != "rotate" {
            continue;
        }
        let closed_at = bindings
            .iter()
            .find(|c| one(g, c, &l("closes")) == Some(b.as_str()))
            .and_then(|c| one(g, c, &pv("generatedAtTime")))
            .map(|t| format!(",valid-before=\"{}\"", keygen_time(t)))
            .unwrap_or_default();
        lines.push(format!(
            "{} namespaces=\"ledger-accept@{}\",valid-after=\"{}\"{closed_at} {} {}",
            strip(one(g, b, &l("principal")), "mailto:").unwrap_or_default(),
            one(g, b, &l("namespace")).unwrap_or_default(),
            keygen_time(one(g, b, &pv("generatedAtTime")).unwrap_or_default()),
            one(g, b, &l("keyType")).unwrap_or_default(),
            one(g, b, &l("publicKey")).unwrap_or_default(),
        ));
    }
    lines.sort();
    format!("{}{}\n", ledger_core::authority::signers::HEADER, lines.join("\n"))
}

/// Every signed entity in the export, verified: `Err` lists the failures.
/// `sig_root` is the directory the export's `ledger:signatureFile` paths
/// are relative to.
pub fn verify_all(g: &Graph, sig_root: &Path) -> Result<usize, Vec<String>> {
    let signers = allowed_signers(g);
    let mut checked = 0;
    let mut failures = Vec::new();
    for (node, preds) in g {
        for sig_node in preds.get(&l("signature")).into_iter().flatten().map(Term::text) {
            checked += 1;
            let file = one(g, sig_node, &l("signatureFile")).unwrap_or_default();
            let sig = std::fs::read(sig_root.join(file)).unwrap_or_default();
            let Some(message) = rebuild(g, node) else {
                failures.push(format!("{node}: not a signable entity"));
                continue;
            };
            let signer = strip(one(g, node, &pv("wasAttributedTo")), "mailto:").unwrap_or_default();
            let ns = namespace_of(g, node).unwrap_or_default();
            let at: chrono::DateTime<chrono::Utc> = one(g, node, &pv("generatedAtTime")).and_then(|t| t.parse().ok()).unwrap_or_default();
            if let Err(e) = ledger_core::signing::ssh::verify(&signers, &signer, &ns, &sig, &message, &at) {
                failures.push(format!("{node}: {e}"));
            }
        }
    }
    if failures.is_empty() { Ok(checked) } else { Err(failures) }
}

/// The ledger namespace an entity is signed in.
fn namespace_of(g: &Graph, node: &str) -> Option<String> {
    if let Some(ns) = one(g, node, &l("namespace")) {
        return Some(ns.to_string());
    }
    let decision = one(g, node, &l("ofDecision")).or_else(|| {
        let acc = one(g, node, &l("revokes"))?;
        one(g, acc, &l("ofDecision"))
    })?;
    decision.strip_prefix("urn:dec:")?.split('/').next().map(str::to_string)
}

//! Which namespace's directory a change-set is filed under (LP-3.34).
//!
//! A file's namespace is its directory, and every entity in it belongs to
//! that namespace (ruling 45). A verb files one act, so the change-set it
//! builds names exactly one namespace: through the decision ids its
//! decisions, versions and acceptances carry; the `namespace` field of a
//! binding or a policy; and, for the records that carry none, the
//! namespace of what they name — a revocation its target's, a grant the
//! grant it is made `under`, a grant acceptance or an interval its grant's.
//! The genesis grant names no `under`; it is filed with the first policy of
//! the namespace it opens.

use std::collections::BTreeSet;

use crate::authority::Revocable;
use crate::changeset::ChangeSet;
use crate::store::Store;

use super::AuthorError;

/// The one namespace `cs` belongs to, read from its entities and, where an
/// entity names a record filed earlier, from `store`.
pub fn namespace_of(store: &Store, cs: &ChangeSet) -> Result<String, AuthorError> {
    let mut found: BTreeSet<String> = BTreeSet::new();
    found.extend(cs.policies.iter().map(|p| p.namespace.clone()));
    found.extend(cs.key_bindings.iter().map(|b| b.namespace.clone()));
    found.extend(cs.decisions.iter().map(|d| d.id.namespace().to_string()));
    found.extend(cs.versions.iter().map(|v| v.decision.namespace().to_string()));
    found.extend(cs.acceptances.iter().map(|a| a.decision.namespace().to_string()));
    for r in &cs.revocations {
        match r.target() {
            Some(Revocable::Acceptance(acc)) => found.extend(acceptance_namespace(store, &acc.to_string())),
            Some(Revocable::Grant(g)) => found.extend(grant_namespace(store, &g.to_string())),
            None => {}
        }
    }
    for g in &cs.grants {
        if let Some(under) = &g.under {
            found.extend(grant_namespace(store, &under.to_string()));
        }
    }
    for ga in &cs.grant_acceptances {
        found.extend(grant_namespace(store, &ga.grant.to_string()));
    }
    for u in &cs.unavailabilities {
        found.extend(grant_namespace(store, &u.grant.to_string()));
    }
    for av in &cs.availabilities {
        found.extend(interval_namespace(store, &av.ends.to_string()));
    }
    let mut it = found.into_iter();
    match (it.next(), it.next()) {
        (Some(ns), None) => Ok(ns),
        (None, _) => Err(AuthorError::Usage(
            "the change-set names no namespace — every entity belongs to one namespace's directory (LP-3.34)".to_string(),
        )),
        (Some(a), Some(b)) => Err(AuthorError::Usage(format!(
            "the change-set holds entities of more than one namespace (`{a}`, `{b}`, …) — a file holds one namespace's entities (ruling 45)"
        ))),
    }
}

/// The namespace whose directory holds the grant `id`.
pub fn grant_namespace(store: &Store, id: &str) -> Option<String> {
    store.log.iter().find(|l| l.file.grants.iter().any(|g| g.id.to_string() == id)).map(|l| l.namespace.clone())
}

/// The namespace of the acceptance `id`: its decision's.
pub fn acceptance_namespace(store: &Store, id: &str) -> Option<String> {
    store
        .log
        .iter()
        .flat_map(|l| l.file.acceptances.iter())
        .find(|a| a.id.to_string() == id)
        .map(|a| a.decision.namespace().to_string())
}

/// The namespace whose directory holds the unavailability `id`.
pub fn interval_namespace(store: &Store, id: &str) -> Option<String> {
    store.log.iter().find(|l| l.file.unavailabilities.iter().any(|u| u.id.to_string() == id)).map(|l| l.namespace.clone())
}

/// The namespace a file is declared in when the verb takes none: the one
/// namespace the store has a directory for. None, or several, is a usage
/// error naming what to pass.
pub fn resolve_namespace_dir(root: &std::path::Path, explicit: Option<&str>) -> Result<String, AuthorError> {
    if let Some(ns) = explicit {
        crate::id::validate_namespace(ns).map_err(AuthorError::Usage)?;
        return Ok(ns.to_string());
    }
    let present = crate::layout::namespaces(root);
    match present.as_slice() {
        [one] => Ok(one.clone()),
        [] => Err(AuthorError::Usage(
            "the store has no namespace directory yet, so there is none to infer — pass --namespace".to_string(),
        )),
        several => Err(AuthorError::Usage(format!(
            "the store holds several namespaces ({}) — pass --namespace",
            several.join(", ")
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit;

    #[test]
    fn a_change_set_is_homed_by_its_decisions_namespace() {
        let cs = testkit::changeset(vec![testkit::sealed(testkit::version())], Vec::new());
        let store = testkit::store(cs.clone());
        assert_eq!(namespace_of(&store, &cs).expect("home"), testkit::NS);
    }

    #[test]
    fn an_empty_change_set_has_no_home_and_two_namespaces_are_refused() {
        let store = testkit::store(testkit::changeset(Vec::new(), Vec::new()));
        let empty = testkit::changeset(Vec::new(), Vec::new());
        assert!(namespace_of(&store, &empty).is_err());
        let mut other = testkit::version();
        other.decision = "dec:hafeok.other/01K2C4YQJ3F8M0PT5W7NZ9RDY0".parse().expect("id");
        let mixed = testkit::changeset(vec![testkit::sealed(testkit::version()), testkit::sealed(other)], Vec::new());
        let err = namespace_of(&store, &mixed).expect_err("two namespaces");
        assert!(err.to_string().contains("ruling 45"), "{err}");
    }

    #[test]
    fn the_namespace_directory_is_inferred_only_when_there_is_one() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert!(resolve_namespace_dir(dir.path(), None).is_err());
        std::fs::create_dir_all(crate::layout::namespace_dir(dir.path(), "a.ns")).expect("mkdir");
        assert_eq!(resolve_namespace_dir(dir.path(), None).expect("one"), "a.ns");
        std::fs::create_dir_all(crate::layout::namespace_dir(dir.path(), "b.ns")).expect("mkdir");
        assert!(resolve_namespace_dir(dir.path(), None).is_err());
        assert_eq!(resolve_namespace_dir(dir.path(), Some("b.ns")).expect("explicit"), "b.ns");
        assert!(resolve_namespace_dir(dir.path(), Some("Not A Namespace")).is_err());
    }
}

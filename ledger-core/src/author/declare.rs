//! `ledger declare` — bringing a decision set into existence.
//!
//! A set fixes the tolerance floor, the ground state, and the owner; it
//! never lists members (§5.2 of the ledger protocol). Declaring one writes a fresh
//! `ns/<ns>/sets/<id>.yml` through the same schema type the loader parses,
//! so a declared set cannot disagree with what `verify` will read back. A
//! set belongs to the namespace whose directory holds it (LP-3.34), and a
//! version names a set of its own namespace (LP-5.22).

use crate::set::{DecisionSet, Ground};
use crate::tier::Tier;

use super::{Applied, Author, AuthorError};

/// What a declaration states.
pub struct DeclareArgs {
    /// The namespace the set is declared in: explicit, else the one
    /// namespace the store has a directory for.
    pub namespace: Option<String>,
    pub id: String,
    pub title: String,
    pub tolerance_floor: Tier,
    pub ground: Ground,
    /// Defaults to the author's own identity when absent.
    pub owner: Option<crate::identity::Identity>,
    pub notes: Option<String>,
}

impl Author {
    /// Declare a new set. Refuses an invalid or already-declared id with
    /// the same wording the parse gate uses.
    pub fn declare(&mut self, args: DeclareArgs) -> Result<Applied, AuthorError> {
        DecisionSet::validate_id(&args.id).map_err(AuthorError::Usage)?;
        let store = self.load();
        let namespace = super::home::resolve_namespace_dir(&self.root, args.namespace.as_deref())?;
        if store.set_in(&namespace, &args.id).is_some() {
            return Err(AuthorError::Usage(format!(
                "set `{}` is already declared under `{namespace}`'s sets/ — a floor change is a set-level governed act, not a redeclaration",
                args.id
            )));
        }
        let set = DecisionSet {
            format: crate::format::CURRENT_FORMAT,
            id: args.id.clone(),
            namespace: namespace.clone(),
            title: args.title,
            tolerance_floor: args.tolerance_floor,
            ground: args.ground,
            owner: args.owner.unwrap_or_else(|| self.who.clone()),
            created_at: self.today(),
            notes: args.notes,
        };
        let dir = crate::layout::sets_dir(&self.root, &namespace);
        std::fs::create_dir_all(&dir).map_err(|e| AuthorError::Io(e.to_string()))?;
        let path = dir.join(format!("{}.yml", set.id));
        if path.exists() {
            return Err(AuthorError::Io(format!("{} already exists", path.display())));
        }
        let text = serde_yaml::to_string(&set)
            .map_err(|e| AuthorError::Io(format!("could not serialise the set: {e}")))?;
        product_core::fileops::write_file_atomic(&path, &text)
            .map_err(|e| AuthorError::Io(e.to_string()))?;
        Ok(Applied {
            path,
            lines: vec![format!(
                "declared set `{}` in `{namespace}` — floor {}, owner {}",
                set.id, set.tolerance_floor, set.owner
            )],
        })
    }
}

//! One decision, on one screen — the read a signature is given before it.
//!
//! `status` answers "where does everything stand"; `blame` answers "who
//! signed what". Neither answers the question an acceptance pass actually
//! asks, which is *what am I signing on this one*. That question needs the
//! hashed content, the edges (ground and reopen, kept apart) **with the
//! argument each edge carries**, the filing act that put it there, and the
//! acceptance situation — assembled from the same [`View`] every other
//! reading uses, so `show` can never disagree with the gate about state.
//!
//! A pointer's type is not the thing being ruled on. `mandate:…` says an
//! edge is a mandate; it does not say what the mandate *is*, and the
//! argument is what a principal actually weighs. Ledger-core cannot read
//! the stores those pointers land in without growing a dependency on every
//! adopter's ontology, so it takes the resolved text as an input: a caller
//! that knows a scheme supplies the argument through [`BasisText`], and a
//! caller that does not passes [`NoText`] and gets bare pointers.
//!
//! The screen is a projection, never an instruction: it says what the
//! decision is and what its state is, and it names the command that would
//! sign it. Filing the signature is the principal's act (OD-3), so nothing
//! here performs one.

use std::fmt;

use chrono::NaiveDate;
use serde::Serialize;

use crate::id::DecisionId;
use crate::store::Store;
use crate::verify::state::{self, DispositionState};
use crate::verify::view::View;

/// Resolves a pointer to the argument it carries. Implemented by whoever
/// owns the store the pointer names; ledger-core owns none of them.
pub trait BasisText {
    /// The argument behind `pointer`, when this resolver knows the scheme.
    fn text(&self, pointer: &str) -> Option<String>;
}

/// The resolver that knows nothing — pointers render bare.
pub struct NoText;

impl BasisText for NoText {
    fn text(&self, _pointer: &str) -> Option<String> {
        None
    }
}

/// Which decisions a `show` covers. A pass over 79 entries is not 79
/// invocations: grouping is the whole point, so the selector is part of
/// the read rather than something a shell loop reconstructs.
pub enum Selector {
    /// Exactly one decision.
    One(DecisionId),
    /// Every decision in one set.
    Set(String),
    /// Every decision whose weight-class matches (see [`Group`]).
    Group(Group),
    /// Every decision the store holds.
    All,
}

/// The selector's canonical spelling — one token, so it can go inside a
/// manifest digest without a second serialisation deciding what a selector
/// "is". A manifest signed over `group:mechanical` cannot be presented for
/// `set:ddd-governance`, however the flags were spelled on the way in.
impl fmt::Display for Selector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::One(id) => write!(f, "decision:{id}"),
            Self::Set(set) => write!(f, "set:{set}"),
            Self::Group(g) => write!(f, "group:{}", g.as_str()),
            Self::All => f.write_str("all"),
        }
    }
}

/// The two weight classes an acceptance pass actually has.
///
/// Derived from content, never hand-listed: a decision is *mechanical* when
/// it is a criterion discharged solely by the repository-diff contract
/// check — the transcribed declarations, where what is being ruled is that
/// a transcription is faithful. Everything else is an individual read.
/// Deriving it means the grouping cannot go stale as entries are added.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    Mechanical,
    Individual,
}

impl Group {
    /// Parse the CLI spelling.
    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "mechanical" => Ok(Self::Mechanical),
            "individual" => Ok(Self::Individual),
            other => Err(format!("unknown group `{other}` — one of mechanical | individual")),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mechanical => "mechanical",
            Self::Individual => "individual",
        }
    }
}

/// One edge as the screen shows it: the pointer, and the argument it
/// carries when a resolver could supply one.
#[derive(Debug, Serialize)]
pub struct Edge {
    pub pointer: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub argument: Option<String>,
}

/// Everything one decision's screen shows. `--json` serialises this whole
/// struct, so the terminal text and the machine form can never carry
/// different facts.
#[derive(Debug, Serialize)]
pub struct Screen {
    pub decision: String,
    pub set: String,
    /// The set's currently declared floor, which is not necessarily the
    /// floor this version pinned — a raise strands members until re-pinned.
    pub set_floor: Option<String>,
    /// The decision's state. For a tip of a forked chain, which has no
    /// latest version (ruling 54), the state that tip's own content and
    /// acceptances give it — one reading per tip, none picked.
    pub state: DispositionState,
    /// Which weight class this entry falls in, so a grouped pass can say
    /// what it is reading.
    pub group: &'static str,
    pub statement: String,
    pub allocation: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub discharge: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discharge_stage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exposure: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_by: Option<String>,
    pub tolerance_floor_at_creation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tolerance_override: Option<String>,
    pub hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// How many versions of this decision the log holds, the latest included.
    pub versions_in_chain: usize,
    /// The ground, each edge with its argument. Never merged with
    /// `revisit_if`: the two mean different things, and a reader deciding
    /// what they sign must see which is which.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub based_on: Vec<Edge>,
    /// The reopen edges — claims whose death reopens this decision.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub revisit_if: Vec<Edge>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub superseded_by: Option<String>,
    /// The act that filed the latest version: change-set, date, author, note.
    pub filed: Filing,
    pub acceptances: Vec<AcceptanceLine>,
    /// True when the chain has more than one tip — nothing may be signed
    /// until `ledger merge --resolve` arbitrates.
    pub forked: bool,
}

/// The change-set that filed the version on screen.
#[derive(Debug, Serialize)]
pub struct Filing {
    pub change_set: String,
    pub at: String,
    pub by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// One acceptance of this decision, and whether it still stands.
#[derive(Debug, Serialize)]
pub struct AcceptanceLine {
    pub id: String,
    pub actor: String,
    pub version: String,
    pub at: String,
    /// `live` | `historical` | `revoked` — the same reading `blame` renders.
    pub standing: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

/// Build the screen for one decision, or `None` when it has no latest
/// version: the log holds none (an unknown id is the caller's error to
/// report, not a blank screen to render), or its chain is forked, whose
/// tips [`screens`] shows one by one.
pub fn screen(
    store: &Store,
    decision: &DecisionId,
    today: NaiveDate,
    texts: &dyn BasisText,
) -> Option<Screen> {
    let view = View::build(store);
    let rows = state::states(&view, today);
    let index = *view.latest.get(&decision.to_string())?;
    build(store, &view, (&rows, today), index, texts)
}

/// Every screen a selector covers, in decision-id order. One `View` build
/// serves the whole pass — the selector exists so a grouped read costs one
/// traversal, not one process per entry.
pub fn screens(
    store: &Store,
    selector: &Selector,
    today: NaiveDate,
    texts: &dyn BasisText,
) -> Vec<Screen> {
    let view = View::build(store);
    let rows = state::states(&view, today);
    // A forked decision has no latest version (ruling 54): each of its tips
    // gets a screen, marked forked, and none is picked as the one.
    let ids: std::collections::BTreeSet<&String> = view.latest.keys().chain(view.forked.keys()).collect();
    ids.into_iter()
        .flat_map(|id| {
            let indices = view.latest.get(id).map(|i| vec![*i]).or_else(|| view.forked.get(id).cloned()).unwrap_or_default();
            indices.into_iter().filter_map(|i| build(store, &view, (&rows, today), i, texts)).collect::<Vec<_>>()
        })
        .filter(|s| matches(s, selector))
        .collect()
}

fn matches(s: &Screen, selector: &Selector) -> bool {
    match selector {
        Selector::One(id) => s.decision == id.to_string(),
        Selector::Set(set) => s.set == *set,
        Selector::Group(g) => s.group == g.as_str(),
        Selector::All => true,
    }
}

/// A criterion whose every discharge pointer is the repository-diff
/// contract check is a transcribed declaration: mechanical. Anything else
/// — a judgment, an escape, a criterion with an analyzer or a test behind
/// it — is an individual read.
fn group_of(raw: &crate::version::VersionRaw) -> Group {
    let mechanical = !raw.discharge.is_empty() && raw.discharge.iter().all(|d| d.is_contract());
    if mechanical {
        Group::Mechanical
    } else {
        Group::Individual
    }
}

fn build(
    store: &Store,
    view: &View,
    (rows, today): (&std::collections::BTreeMap<String, state::StateRow>, NaiveDate),
    index: usize,
    texts: &dyn BasisText,
) -> Option<Screen> {
    let viewed = view.versions.get(index)?;
    let raw = viewed.raw;
    let decision = raw.decision.to_string();
    let id = decision.as_str();
    let row = rows.get(id);
    let filing = store.log.iter().find(|l| l.path == viewed.path).map(|l| &l.file);
    let edges = |items: Vec<String>| -> Vec<Edge> {
        items
            .into_iter()
            .map(|pointer| Edge { argument: texts.text(&pointer), pointer })
            .collect()
    };
    Some(Screen {
        decision: id.to_string(),
        set: raw.set.clone(),
        set_floor: store.set_in(raw.decision.namespace(), &raw.set).map(|s| s.tolerance_floor.to_string()),
        state: match (row, viewed.parsed.as_ref()) {
            (Some(r), _) => r.state,
            (None, Some(tip)) if view.is_forked(id) => state::tip_state(view, tip, today),
            (None, _) => DispositionState::Undecided,
        },
        group: group_of(raw).as_str(),
        statement: raw.statement.clone(),
        allocation: raw.allocation.map(|a| a.as_str().to_string()),
        discharge: raw.discharge.iter().map(ToString::to_string).collect(),
        discharge_stage: raw.discharge_stage.map(|s| s.to_string()),
        actor: raw.actor.as_ref().map(ToString::to_string),
        exposure: raw.exposure.clone(),
        review_by: raw.review_by.map(|d| d.to_string()),
        tolerance_floor_at_creation: raw.tolerance_floor_at_creation.to_string(),
        tolerance_override: raw.tolerance_override.map(|t| t.to_string()),
        hash: raw.hash.to_string(),
        parent: raw.parent.as_ref().map(ToString::to_string),
        versions_in_chain: view
            .versions
            .iter()
            .filter(|v| v.raw.decision.to_string() == id)
            .count(),
        based_on: edges(raw.based_on.iter().map(ToString::to_string).collect()),
        revisit_if: edges(raw.revisit_if.iter().map(ToString::to_string).collect()),
        supersedes: raw.supersedes.as_ref().map(ToString::to_string),
        superseded_by: row.and_then(|r| r.superseded_by.clone()),
        filed: Filing {
            change_set: filing.map(|f| f.id.to_string()).unwrap_or_default(),
            at: filing.map(|f| f.created_at.date_naive().to_string()).unwrap_or_default(),
            by: filing.map(|f| f.created_by.to_string()).unwrap_or_default(),
            note: filing.and_then(|f| f.note.clone()),
        },
        acceptances: acceptance_lines(view, id),
        forked: view.is_forked(id),
    })
}

fn acceptance_lines(view: &View, id: &str) -> Vec<AcceptanceLine> {
    view.acceptances
        .iter()
        .map(|v| v.acceptance)
        .filter(|a| a.decision.to_string() == id)
        .map(|a| AcceptanceLine {
            id: a.id.to_string(),
            actor: a.actor.to_string(),
            version: a.version.short().to_string(),
            at: a.at.date_naive().to_string(),
            standing: if view.is_revoked(a) {
                "revoked"
            } else if view.signs_a_tip(a) {
                // A tip of a forked chain counts: nothing is picked among tips.
                "live"
            } else {
                "historical"
            },
            expires_at: a.expires_at.map(|d| d.to_string()),
        })
        .collect()
}

#[path = "show_render.rs"]
mod render;
pub use render::{render, render_all};

#[path = "show_tests.rs"]
#[cfg(test)]
mod tests;

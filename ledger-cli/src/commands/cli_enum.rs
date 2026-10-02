//! The `ledger` subcommand surface, as clap reads it.

use std::path::PathBuf;

use clap::Subcommand;

/// The command surface. Keep the variant list sorted.
#[derive(Subcommand)]
pub enum Commands {
    /// Sign the latest version of a decision, or of a whole read selection
    /// (identity from git config; one acceptance record per decision)
    Accept {
        /// The decision id, `dec:<namespace>/<ulid>`
        decision: Option<String>,
        /// Every decision in one set — enumerated, then confirmed
        #[arg(long, value_name = "SET")]
        set: Option<String>,
        /// Every decision in one weight class: mechanical | individual
        #[arg(long, value_name = "GROUP")]
        group: Option<String>,
        /// When this signature goes stale (YYYY-MM-DD)
        #[arg(long, value_name = "DATE")]
        expires: Option<String>,
        /// The manifest a dry run printed. Without it nothing is written
        #[arg(long, value_name = "MANIFEST")]
        confirm: Option<String>,
        /// Emit the selection as JSON
        #[arg(long)]
        json: bool,
    },
    /// Mint a decision and file its first version into a set
    Add {
        #[arg(long, value_name = "SET")]
        set: String,
        #[arg(long)]
        statement: String,
        /// Owning scope of the new id; inferred when the store speaks one
        #[arg(long, value_name = "NS")]
        namespace: Option<String>,
        /// Allocation store: constraint | criterion | judgment
        #[arg(long, value_name = "STORE")]
        store: Option<String>,
        /// Discharge pointer(s), e.g. analyzer:DEC001 (repeatable)
        #[arg(long, value_name = "REF")]
        discharge: Vec<String>,
        /// Criterion stage: pr | dev | staging | prod
        #[arg(long, value_name = "STAGE")]
        stage: Option<String>,
        /// Expectation, required when a discharge is otel:
        #[arg(long)]
        expectation: Option<String>,
        /// The named actor, for a judgment
        #[arg(long, value_name = "IDENTITY")]
        actor: Option<String>,
        /// Up-only tolerance override, strictly above the set floor
        #[arg(long, value_name = "TIER")]
        tolerance_override: Option<String>,
        /// Basis pointer(s) this decision rests on (repeatable)
        #[arg(long, value_name = "BASIS")]
        based_on: Vec<String>,
        /// Claim(s) whose death reopens this decision — never ground
        #[arg(long, value_name = "REF")]
        revisit_if: Vec<String>,
        /// What this act was, recorded on the change-set
        #[arg(long)]
        note: Option<String>,
        /// The stable human name a generated type is built from (format 5)
        #[arg(long, value_name = "KEY")]
        key: Option<String>,
        /// Make the decision citable from other namespaces (format 5)
        #[arg(long)]
        exported: bool,
    },
    /// Allocate (or re-allocate) a decision by filing its next version
    Allocate {
        decision: String,
        /// Allocation store: constraint | criterion | judgment
        #[arg(long, value_name = "STORE")]
        store: String,
        #[arg(long, value_name = "REF")]
        discharge: Vec<String>,
        #[arg(long, value_name = "STAGE")]
        stage: Option<String>,
        #[arg(long)]
        expectation: Option<String>,
        #[arg(long, value_name = "IDENTITY")]
        actor: Option<String>,
    },
    /// Who signed which version of a decision, and who committed it
    Blame {
        decision: String,
    },
    /// Disposition coverage per set, per namespace, with supersession chains
    Coverage {
        /// Limit to one set
        #[arg(long, value_name = "SET")]
        set: Option<String>,
        /// Emit the report as JSON
        #[arg(long)]
        json: bool,
        /// Judge expiry against this date instead of today
        #[arg(long, value_name = "DATE")]
        today: Option<String>,
    },
    /// Declare a decision set: floor, ground, owner
    Declare {
        #[arg(long, value_name = "ID")]
        set: String,
        /// Human title; defaults to the set id
        #[arg(long)]
        title: Option<String>,
        #[arg(long, value_name = "TIER")]
        tolerance_floor: String,
        /// characterised | uncharacterised
        #[arg(long, default_value = "characterised")]
        ground: String,
        /// Defaults to the git-config identity
        #[arg(long, value_name = "IDENTITY")]
        owner: Option<String>,
        #[arg(long)]
        notes: Option<String>,
    },
    /// Decision-level change between two revisions (semantic, never textual)
    Diff {
        /// `<ref>..<ref>`, or `<ref>` to compare against the working tree
        spec: String,
        /// Emit the report as JSON
        #[arg(long)]
        json: bool,
    },
    /// Price an escape: exposure stated, review date set, acceptor claimed
    Escape {
        decision: String,
        #[arg(long)]
        exposure: String,
        #[arg(long, value_name = "DATE")]
        review_by: String,
    },
    /// Write the log as sorted, canonical N-Triples, one file per namespace
    Export {
        /// Output format: ntriples
        #[arg(long, value_name = "FORMAT")]
        format: String,
        /// One namespace; default every namespace the log speaks
        #[arg(long, value_name = "NS")]
        namespace: Option<String>,
        /// Output file (default docs/decisions/<ns>.nt); `-` prints it
        #[arg(long, value_name = "PATH")]
        out: Option<PathBuf>,
    },
    /// Scaffold the .decisions/ store: the §5 layout plus the ignore line
    Init,
    /// Walk the change-sets in creation order
    Log {
        /// Only change-sets touching this set
        #[arg(long, value_name = "SET")]
        set: Option<String>,
    },
    /// Reconcile divergent version chains: plan, arbitrate, or install
    Merge {
        /// Revision to plan a merge with (read-only; exit 1 on conflicts)
        rev: Option<String>,
        /// Present each conflict and record the arbitration as a ledger act
        #[arg(long)]
        resolve: bool,
        /// Register the merge driver and .decisions/.gitattributes
        #[arg(long)]
        install: bool,
        /// Emit the plan as JSON
        #[arg(long)]
        json: bool,
    },
    /// The git merge driver: three-way judge one .decisions/ file
    #[command(hide = true)]
    MergeDriver {
        /// %O — the merge ancestor's version of the file
        base: PathBuf,
        /// %A — ours; the result is left here
        ours: PathBuf,
        /// %B — theirs
        theirs: PathBuf,
        /// %P — the repo-relative pathname
        path: String,
    },
    /// Rebuild the RDF index under .decisions/index/ from the log
    Reindex,
    /// Restate a decision as a new version; prior acceptances become history
    Revise {
        decision: String,
        #[arg(long)]
        statement: String,
        #[arg(long, value_name = "BASIS")]
        based_on: Vec<String>,
        /// Reopen edge(s) this version states, replacing the inherited set
        #[arg(long, value_name = "REF")]
        revisit_if: Vec<String>,
        /// Drop every inherited reopen edge (the decision was reopened)
        #[arg(long)]
        no_revisit_if: bool,
        /// What this act was, recorded on the change-set
        #[arg(long)]
        note: Option<String>,
        /// The hash believed to be the tip; refused when stale (merge is L3)
        #[arg(long, value_name = "HASH")]
        parent: Option<String>,
        /// Give a keyless decision its key; a keyed decision keeps its own
        #[arg(long, value_name = "KEY")]
        key: Option<String>,
    },
    /// Reverse a prior acceptance, with the reason on record
    Revoke {
        acceptance: String,
        #[arg(long)]
        reason: String,
    },
    /// Decisions on screen: content, edges with their arguments, filing,
    /// acceptance. One id, or a whole set or group in one pass.
    Show {
        decision: Option<String>,
        /// Every decision in one set
        #[arg(long, value_name = "SET")]
        set: Option<String>,
        /// Every decision in one weight class: mechanical | individual
        #[arg(long, value_name = "GROUP")]
        group: Option<String>,
        /// Emit the screens as JSON
        #[arg(long)]
        json: bool,
        /// Judge acceptance expiry against this date instead of today
        #[arg(long, value_name = "DATE")]
        today: Option<String>,
    },
    /// Every decision's disposition state, grouped by set
    Status {
        /// Judge expiry against this date instead of today
        #[arg(long, value_name = "DATE")]
        today: Option<String>,
    },
    /// Record that one decision supersedes another (never amend)
    Supersede {
        /// The decision being superseded
        decision: String,
        /// The successor decision
        #[arg(long, value_name = "DEC")]
        by: String,
        #[arg(long)]
        reason: Option<String>,
    },
    /// Run the gate over the log (exit 1 on findings, 2 on failure to run)
    Verify {
        /// Limit to one gate: `readiness` blocks produce, `completeness`
        /// blocks release. Default runs every class.
        #[arg(long, value_name = "GATE")]
        gate: Option<String>,
        /// Emit the report as JSON
        #[arg(long)]
        json: bool,
        /// Judge acceptance expiry against this date instead of today
        #[arg(long, value_name = "DATE")]
        today: Option<String>,
        /// Skip the git blame pass (class L009)
        #[arg(long)]
        no_blame: bool,
        /// Also hold every committed docs/decisions/<ns>.nt byte-identical
        /// to the log's export
        #[arg(long)]
        export: bool,
    },
}


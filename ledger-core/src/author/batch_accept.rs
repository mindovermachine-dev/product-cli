//! `ledger accept --batch <file>` — sign exactly the rows a selection file
//! lists (#86).
//!
//! The file is the hand-off `ledger inbox` writes ([`crate::batch_file`]).
//! This clone signs its own rows — those for its repository label, on the
//! checked-out branch — and only against the manifest digest a dry run of
//! the same file printed. Every row is checked against this store first:
//! a decision the store lacks, a version hash that moved, a held or forked
//! chain, a grant that does not qualify under the batch's `--as` or that
//! differs from the one the file names — each refuses the whole batch. The
//! signing itself is `accept --group`'s: one change-set, one acceptance per
//! row, one signature per acceptance, every row through the same gate.

use chrono::NaiveDate;

use crate::batch::{self, Member, Outcome, Plan, Refusal, Standing};
use crate::batch_file::{self, BatchFile, Row, NO_GRANT};
use crate::id::DecisionId;
use crate::show::{screens, NoText, Selector};
use crate::store::Store;

use super::group_accept::refuse;
use super::{Author, AuthorError};

/// One `accept --batch` invocation.
pub struct AcceptBatchArgs {
    pub file: BatchFile,
    /// How the file was named, for the confirm line.
    pub path: String,
    /// The rows this clone signs: the file's single repository when absent.
    pub repository: Option<String>,
    /// The branch checked out here, when there is one.
    pub branch: Option<String>,
    pub expires_at: Option<NaiveDate>,
    pub confirm: Option<String>,
}

impl Author {
    /// Resolve this clone's rows, and — only against a matching manifest —
    /// sign them.
    pub fn accept_batch(&mut self, args: AcceptBatchArgs) -> Result<Outcome, AuthorError> {
        let repository = self.admit(&args)?;
        let store = self.load();
        let today = self.today();
        let mine: Vec<&Row> = args.file.rows.iter().filter(|r| r.is_for(&repository, None)).collect();
        let (members, problems) = self.resolve(&store, &mine, today);
        let mut resolved = args.file.clone();
        for row in resolved.rows.iter_mut().filter(|r| r.is_for(&repository, None)) {
            if let Some(m) = members.iter().find(|m| m.decision == row.decision) {
                row.grant = m.grant.clone().filter(|g| g != NO_GRANT);
            }
        }
        let plan = Plan {
            selector: confirm_flags(&args.path, &args.file, &repository),
            actor: self.who.to_string(),
            members,
            manifest: batch_file::manifest(&resolved),
        };
        if let Some(refusal) = problems {
            return Ok(refuse(plan, refusal));
        }
        let Some(confirmed) = args.confirm else {
            return Ok(Outcome { plan, dry_run: true, signed: Vec::new(), filed: None, refusal: None });
        };
        if !batch::is_manifest_shaped(&confirmed) {
            return Err(AuthorError::Usage(format!(
                "`{confirmed}` is not a manifest — expected the `sha256:<64 hex>` digest a dry run prints"
            )));
        }
        if confirmed != plan.manifest {
            let message = format!(
                "the batch moved since it was read — you confirmed {confirmed}, this file and store now pin {}",
                plan.manifest
            );
            return Ok(refuse(plan, Refusal { kind: "drift", message, detail: Vec::new() }));
        }
        self.sign(&store, plan, args.expires_at)
    }

    /// Whose batch, which rows, which role: the refusals before any row is
    /// read. Returns the repository label this clone signs for.
    fn admit(&mut self, args: &AcceptBatchArgs) -> Result<String, AuthorError> {
        if args.file.actor != self.who.to_string() {
            return Err(AuthorError::Unauthorized(format!(
                "this batch is {}'s; you are {} — no one accepts on another principal's behalf",
                args.file.actor, self.who
            )));
        }
        let repository = repository_of(&args.file, args.repository.as_deref())?;
        if args.as_role_differs(self.as_role.as_deref()) {
            return Err(AuthorError::Usage(format!(
                "the batch is made as `{}` — `--as` comes from the file, not the command line",
                args.file.as_role.clone().unwrap_or_default()
            )));
        }
        self.as_role = args.file.as_role.clone();
        let mine = args.file.rows.iter().filter(|r| r.is_for(&repository, None));
        if let Some(wrong) = mine.into_iter().find(|r| r.branch.is_some() && args.branch.is_some() && r.branch != args.branch) {
            return Err(AuthorError::Usage(format!(
                "{} is on branch `{}`, but `{}` is checked out — sign each branch's rows on that branch",
                wrong.decision,
                wrong.branch.clone().unwrap_or_default(),
                args.branch.clone().unwrap_or_default()
            )));
        }
        Ok(repository)
    }

    /// Each row against this store: present, at its hash, signable, and
    /// under a grant that qualifies. Problems are collected, not raised, so
    /// the whole batch is shown with every reason it cannot be signed.
    fn resolve(&self, store: &Store, rows: &[&Row], today: NaiveDate) -> (Vec<Member>, Option<Refusal>) {
        let view = crate::verify::view::View::build(store);
        let mut members = Vec::new();
        let (mut drift, mut other) = (Vec::new(), Vec::new());
        for row in rows {
            let Ok(decision) = row.decision.parse::<DecisionId>() else {
                other.push(format!("{} — not a decision id", row.decision));
                continue;
            };
            let selector = Selector::One(decision.clone());
            let read = batch::plan(&screens(store, &selector, today, &NoText), &selector, &self.who, today);
            let Some(mut m) = read.members.into_iter().next() else {
                drift.push(format!("{} — missing: this store holds no such decision", row.decision));
                continue;
            };
            if m.version != row.version {
                drift.push(format!("{} — moved: the batch signs {}, the latest version is {}", row.decision, row.version, m.version));
            }
            if m.standing == Standing::Held.as_str() || m.standing == Standing::Forked.as_str() {
                other.push(format!("{} — {}", row.decision, m.because.clone().unwrap_or_default()));
            } else if let Err(e) = self.refuse_duplicate(&view, &decision, &m.version.parse().unwrap_or_else(|_| crate::hash::VersionHash::zero())) {
                other.push(format!("{} — {e}", row.decision));
            }
            let grant = match self.decision_authority(store, &view, &decision, crate::authority::Act::Accept) {
                Ok(held) => held.map(|a| a.grant).unwrap_or_else(|| NO_GRANT.to_string()),
                Err(e) => {
                    other.push(format!("{} — {e}", row.decision));
                    NO_GRANT.to_string()
                }
            };
            if row.grant.as_deref().is_some_and(|g| g != grant) {
                drift.push(format!("{} — the batch names grant {}, this store resolves {grant}", row.decision, row.grant.clone().unwrap_or_default()));
            }
            m.standing = Standing::Signable.as_str();
            m.because = None;
            m.repository = Some(match &row.branch {
                Some(b) => format!("{} @ {b}", row.repository),
                None => row.repository.clone(),
            });
            m.grant = Some(grant);
            members.push(m);
        }
        (members, refusal(drift, other))
    }
}

impl AcceptBatchArgs {
    fn as_role_differs(&self, cli: Option<&str>) -> bool {
        cli.is_some() && cli != self.file.as_role.as_deref()
    }
}

/// The repository label this clone signs for.
fn repository_of(file: &BatchFile, named: Option<&str>) -> Result<String, AuthorError> {
    let labels: std::collections::BTreeSet<&str> = file.rows.iter().map(|r| r.repository.as_str()).collect();
    match named {
        Some(r) if labels.contains(r) => Ok(r.to_string()),
        Some(r) => Err(AuthorError::Usage(format!("the batch lists no row for repository `{r}`"))),
        None if labels.len() == 1 => Ok(labels.into_iter().next().unwrap_or_default().to_string()),
        None => Err(AuthorError::Usage(format!(
            "the batch spans {} repositories — name this clone's with `--repository`",
            labels.len()
        ))),
    }
}

fn confirm_flags(path: &str, file: &BatchFile, repository: &str) -> String {
    let spans = file.rows.iter().any(|r| r.repository != repository);
    if spans {
        format!("--batch {path} --repository {repository}")
    } else {
        format!("--batch {path}")
    }
}

fn refusal(drift: Vec<String>, other: Vec<String>) -> Option<Refusal> {
    if !drift.is_empty() {
        let mut detail = drift;
        detail.extend(other);
        return Some(Refusal {
            kind: "drift",
            message: "the batch no longer matches this store — it signs exactly the rows it lists, at exactly their hashes".to_string(),
            detail,
        });
    }
    (!other.is_empty()).then(|| Refusal {
        kind: "row",
        message: "a row of this batch cannot be accepted as it stands — the whole batch is refused rather than signed around it"
            .to_string(),
        detail: other,
    })
}

